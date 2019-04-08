
import re
import sys
import csv
import json
import ftfy
import itertools
import subprocess
from collections import Counter
from confluent_kafka import Consumer, KafkaException

import fatcat_client
from fatcat_client.rest import ApiException


def clean(thing, force_xml=False):
    """
    This function is appropriate to be called on any random, non-markup string,
    such as author names, titles, etc.

    It will try to clean up commong unicode mangles, HTML characters, etc.

    This will detect XML/HTML and "do the right thing" (aka, not remove
    entities like '&amp' if there are tags in the string), unless you pass the
    'force_xml' parameter, which might be appropriate for, eg, names and
    titles, which generally should be projected down to plain text.

    Also strips extra whitespace.
    """
    if not thing:
        return None
    fix_entities = 'auto'
    if force_xml:
        fix_entities = True
    fixed = ftfy.fix_text(thing, fix_entities=fix_entities).strip()
    if not fixed or len(fixed) <= 1:
        # wasn't zero-length before, but is now; return None
        return None
    return fixed

def test_clean():

    assert clean(None) == None
    assert clean('') == None
    assert clean('1') == None
    assert clean('123') == '123'
    assert clean('a&amp;b') == 'a&b'
    assert clean('<b>a&amp;b</b>') == '<b>a&amp;b</b>'
    assert clean('<b>a&amp;b</b>', force_xml=True) == '<b>a&b</b>'

DOMAIN_REL_MAP = {
    "archive.org": "repository",
    "arxiv.org": "repository",
    "babel.hathitrust.org": "repository",
    "cds.cern.ch": "repository",
    "citeseerx.ist.psu.edu": "repository",
    "deepblue.lib.umich.edu": "repository",
    "europepmc.org": "repository",
    "hal.inria.fr": "repository",
    "scielo.isciii.es": "repository",
    "www.dtic.mil": "repository",
    "www.jstage.jst.go.jp": "repository",
    "www.jstor.org": "repository",
    "www.ncbi.nlm.nih.gov": "repository",
    "www.scielo.br": "repository",
    "www.scielo.cl": "repository",
    "www.scielo.org.mx": "repository",
    "zenodo.org": "repository",

    "academic.oup.com": "publisher",
    "cdn.elifesciences.org": "publisher",
    "cell.com": "publisher",
    "dl.acm.org": "publisher",
    "downloads.hindawi.com": "publisher",
    "elifesciences.org": "publisher",
    "iopscience.iop.org": "publisher",
    "journals.plos.org": "publisher",
    "link.springer.com": "publisher",
    "onlinelibrary.wiley.com": "publisher",
    "works.bepress.com": "publisher",
    "www.biomedcentral.com": "publisher",
    "www.cell.com": "publisher",
    "www.nature.com": "publisher",
    "www.pnas.org": "publisher",
    "www.tandfonline.com": "publisher",

    "www.researchgate.net": "social",
    "academia.edu": "social",

    "wayback.archive-it.org": "webarchive",
    "web.archive.org": "webarchive",
    "archive.is": "webarchive",
}

def make_rel_url(raw_url, default_link_rel="web"):
    # this is where we map specific domains to rel types, and also filter out
    # bad domains, invalid URLs, etc
    rel = default_link_rel
    for domain, domain_rel in DOMAIN_REL_MAP.items():
        if "//{}/".format(domain) in raw_url:
            rel = domain_rel
            break
    return (rel, raw_url)

def test_make_rel_url():
    assert make_rel_url("http://example.com/thing.pdf")[0] == "web"
    assert make_rel_url("http://example.com/thing.pdf", default_link_rel="jeans")[0] == "jeans"
    assert make_rel_url("https://web.archive.org/web/*/http://example.com/thing.pdf")[0] == "webarchive"
    assert make_rel_url("http://cell.com/thing.pdf")[0] == "publisher"

class EntityImporter:
    """
    Base class for fatcat entity importers.

    The API exposed to record iterator is:

        push_record(raw_record)
        finish()

    The API that implementations are expected to fill in are:

        want(raw_record) -> boolean
        parse(raw_record) -> entity
        try_update(entity) -> boolean
        insert_batch([entity]) -> None

    This class exposes helpers for implementations:

        self.api
        self.get_editgroup_id()
        self.create_<entity>(entity) -> EntityEdit
            for related entity types
        self.push_entity(entity)
        self.counts['exists'] += 1
            if didn't update or insert because of existing)
        self.counts['update'] += 1
            if updated an entity
    """

    def __init__(self, api, **kwargs):

        eg_extra = kwargs.get('editgroup_extra', dict())
        eg_extra['git_rev'] = eg_extra.get('git_rev',
            subprocess.check_output(["git", "describe", "--always"]).strip()).decode('utf-8')
        eg_extra['agent'] = eg_extra.get('agent', 'fatcat_tools.EntityImporter')

        self.api = api
        self.bezerk_mode = kwargs.get('bezerk_mode', False)
        self.edit_batch_size = kwargs.get('edit_batch_size', 100)
        self.editgroup_description = kwargs.get('editgroup_description')
        self.editgroup_extra = kwargs.get('editgroup_extra')
        self.reset()

        self._issnl_id_map = dict()
        self._orcid_id_map = dict()
        self._orcid_regex = re.compile("^\\d{4}-\\d{4}-\\d{4}-\\d{3}[\\dX]$")
        self._doi_id_map = dict()

    def reset(self):
        self.counts = Counter({'skip': 0, 'insert': 0, 'update': 0, 'exists': 0})
        self._edit_count = 0
        self._editgroup_id = None
        self._entity_queue = []

    def push_record(self, raw_record):
        """
        Returns nothing.
        """
        if (not raw_record) or (not self.want(raw_record)):
            self.counts['skip'] += 1
            return
        entity = self.parse_record(raw_record)
        if not entity:
            self.counts['skip'] += 1
            return
        if self.bezerk_mode:
            self.push_entity(entity)
            return
        if self.try_update(entity):
            self.push_entity(entity)
        return

    def parse_record(self, raw_record):
        # implementations should fill this in
        raise NotImplementedError

    def finish(self):
        if self._edit_count > 0:
            self.api.accept_editgroup(self._editgroup_id)
            self._editgroup_id = None
            self._edit_count = 0

        if self._entity_queue:
            self.insert_batch(self._entity_queue)
            self.counts['insert'] += len(self._entity_queue)
            self._entity_queue =  []

        self.counts['total'] = 0
        for key in ('skip', 'insert', 'update', 'exists'):
            self.counts['total'] += self.counts[key]
        return self.counts

    def get_editgroup_id(self, edits=1):
        if self._edit_count >= self.edit_batch_size:
            self.api.accept_editgroup(self._editgroup_id)
            self._editgroup_id = None
            self._edit_count = 0

        if not self._editgroup_id:
            eg = self.api.create_editgroup(
                fatcat_client.Editgroup(
                    description=self.editgroup_description,
                    extra=self.editgroup_extra))
            self._editgroup_id = eg.editgroup_id

        self._edit_count += edits
        return self._editgroup_id

    def create_container(self, entity):
        eg_id = self.get_editgroup_id()
        self.counts['inserted.container'] += 1
        return self.api.create_container(entity, editgroup_id=eg_id)

    def create_release(self, entity):
        eg_id = self.get_editgroup_id()
        self.counts['inserted.release'] += 1
        return self.api.create_release(entity, editgroup_id=eg_id)

    def create_file(self, entity):
        eg_id = self.get_editgroup_id()
        self.counts['inserted.file'] += 1
        return self.api.create_file(entity, editgroup_id=eg_id)

    def updated(self):
        """
        Implementations should call this from try_update() if the update was successful
        """
        self.counts['update'] += 1

    def push_entity(self, entity):
        self._entity_queue.append(entity)
        if len(self._entity_queue) >= self.edit_batch_size:
            self.insert_batch(self._entity_queue)
            self.counts['insert'] += len(self._entity_queue)
            self._entity_queue = []

    def want(self, raw_record):
        """
        Implementations can override for optional fast-path to drop a record.
        Must have no side-effects; returns bool.
        """
        return True

    def parse(self, raw_record):
        """
        Returns an entity class type, or None if we should skip this one.

        May have side-effects (eg, create related entities), but shouldn't
        update/mutate the actual entity.
        """
        raise NotImplementedError

    def try_update(self, raw_record):
        """
        Passed the output of parse(). Should try to find an existing entity and
        update it (PUT), decide we should do nothing (based on the existing
        record), or create a new one.

        Implementations must update the exists/updated/skip counts
        appropriately in this method.

        Returns boolean: True if the entity should still be inserted, False otherwise
        """
        raise NotImplementedError

    def insert_batch(self, raw_record):
        raise NotImplementedError

    def is_orcid(self, orcid):
        return self._orcid_regex.match(orcid) is not None

    def lookup_orcid(self, orcid):
        """Caches calls to the Orcid lookup API endpoint in a local dict"""
        if not self.is_orcid(orcid):
            return None
        if orcid in self._orcid_id_map:
            return self._orcid_id_map[orcid]
        creator_id = None
        try:
            rv = self.api.lookup_creator(orcid=orcid)
            creator_id = rv.ident
        except ApiException as ae:
            # If anything other than a 404 (not found), something is wrong
            assert ae.status == 404
        self._orcid_id_map[orcid] = creator_id # might be None
        return creator_id

    def is_doi(self, doi):
        return doi.startswith("10.") and doi.count("/") >= 1

    def lookup_doi(self, doi):
        """Caches calls to the doi lookup API endpoint in a local dict"""
        assert self.is_doi(doi)
        doi = doi.lower()
        if doi in self._doi_id_map:
            return self._doi_id_map[doi]
        release_id = None
        try:
            rv = self.api.lookup_release(doi=doi)
            release_id = rv.ident
        except ApiException as ae:
            # If anything other than a 404 (not found), something is wrong
            assert ae.status == 404
        self._doi_id_map[doi] = release_id # might be None
        return release_id

    def is_issnl(self, issnl):
        return len(issnl) == 9 and issnl[4] == '-'

    def lookup_issnl(self, issnl):
        """Caches calls to the ISSN-L lookup API endpoint in a local dict"""
        if issnl in self._issnl_id_map:
            return self._issnl_id_map[issnl]
        container_id = None
        try:
            rv = self.api.lookup_container(issnl=issnl)
            container_id = rv.ident
        except ApiException as ae:
            # If anything other than a 404 (not found), something is wrong
            assert ae.status == 404
        self._issnl_id_map[issnl] = container_id # might be None
        return container_id

    def read_issn_map_file(self, issn_map_file):
        print("Loading ISSN map file...")
        self._issn_issnl_map = dict()
        for line in issn_map_file:
            if line.startswith("ISSN") or len(line) == 0:
                continue
            (issn, issnl) = line.split()[0:2]
            self._issn_issnl_map[issn] = issnl
            # double mapping makes lookups easy
            self._issn_issnl_map[issnl] = issnl
        print("Got {} ISSN-L mappings.".format(len(self._issn_issnl_map)))

    def issn2issnl(self, issn):
        if issn is None:
            return None
        return self._issn_issnl_map.get(issn)


class RecordPusher:
    """
    Base class for different importer sources. Pretty trivial interface, just
    wraps an importer and pushes records in to it.
    """

    def __init__(self, importer, **kwargs):
        self.importer = importer

    def run(self):
        """
        This will look something like:

            for line in sys.stdin:
                record = json.loads(line)
                self.importer.push_record(record)
            print(self.importer.finish())
        """
        raise NotImplementedError


class JsonLinePusher(RecordPusher):

    def __init__(self, importer, json_file, **kwargs):
        self.importer = importer
        self.json_file = json_file

    def run(self):
        for line in self.json_file:
            if not line:
                continue
            record = json.loads(line)
            self.importer.push_record(record)
        counts = self.importer.finish()
        print(counts)
        return counts


class CsvPusher(RecordPusher):

    def __init__(self, importer, csv_file, **kwargs):
        self.importer = importer
        self.reader = csv.DictReader(csv_file, delimiter=kwargs.get('delimiter', ','))

    def run(self):
        for line in self.reader:
            if not line:
                continue
            self.importer.push_record(line)
        counts = self.importer.finish()
        print(counts)
        return counts


class LinePusher(RecordPusher):

    def __init__(self, importer, text_file, **kwargs):
        self.importer = importer
        self.text_file = text_file

    def run(self):
        for line in self.text_file:
            if not line:
                continue
            self.importer.push_record(line)
        counts = self.importer.finish()
        print(counts)
        return counts


class KafkaJsonPusher(RecordPusher):

    def __init__(self, importer, kafka_hosts, kafka_env, topic_suffix, group, **kwargs):
        self.importer = importer
        self.consumer = make_kafka_consumer(
            kafka_hosts,
            kafka_env,
            topic_suffix,
            group,
        )
        self.edit_batch_size = kwargs.get('edit_batch_size', 100)

    def run(self):
        count = 0
        while True:
            batch = self.consumer.consume(
                num_messages=self.edit_batch_size,
                timeout=3.0)
            print("... got {} kafka messages".format(len(batch)))
            if not batch:
                # TODO: could have some larger timeout here and
                # self.importer.finish() if it's been more than, eg, a couple
                # minutes
                continue
            # first check errors on entire batch...
            for msg in batch:
                if msg.error():
                    raise KafkaException(msg.error())
            # ... then process
            for msg in batch:
                record = json.loads(msg.value().decode('utf-8'))
                self.importer.push_record(record)
                count += 1
                if count % 500 == 0:
                    print("Import counts: {}".format(self.importer.counts))
            # locally store the last processed message; will be auto-commited
            # from this "stored" value
            assert msg
            self.consumer.store_offsets(msg)
        # TODO: should catch UNIX signals (HUP?) to shutdown cleanly, and/or
        # commit the current batch if it has been lingering
        counts = self.importer.finish()
        print(counts)
        self.consumer.close()
        return counts


def make_kafka_consumer(hosts, env, topic_suffix, group):
    topic_name = "fatcat-{}.{}".format(env, topic_suffix)

    def fail_fast(err, partitions):
        if err is not None:
            print("Kafka consumer commit error: {}".format(err))
            print("Bailing out...")
            # TODO: should it be sys.exit(-1)?
            raise KafkaException(err)
        for p in partitions:
            # check for partition-specific commit errors
            print(p)
            if p.error:
                print("Kafka consumer commit error: {}".format(p.error))
                print("Bailing out...")
                # TODO: should it be sys.exit(-1)?
                raise KafkaException(err)
        #print("Kafka consumer commit successful")
        pass

    # previously, using pykafka
    #auto_commit_enable=True,
    #auto_commit_interval_ms=30000, # 30 seconds
    conf = {
        'bootstrap.servers': hosts,
        'group.id': group.encode('utf-8'),
        'on_commit': fail_fast,
        'delivery.report.only.error': True,
        'enable.auto.offset.store': False,
        'default.topic.config': {
            'request.required.acks': -1,
            'auto.offset.reset': 'latest',
        },
    }

    def on_rebalance(consumer, partitions):
        print("Kafka partitions rebalanced: {} / {}".format(
            consumer, partitions))

    consumer = Consumer(conf)
    # NOTE: it's actually important that topic_name *not* be bytes (UTF-8
    # encoded)
    consumer.subscribe([topic_name],
        on_assign=on_rebalance,
        on_revoke=on_rebalance,
    )
    print("Consuming from kafka topic {}, group {}".format(topic_name, group))
    return consumer
