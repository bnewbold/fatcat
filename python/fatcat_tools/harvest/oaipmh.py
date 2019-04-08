
import re
import sys
import csv
import json
import time
import itertools
import datetime
import requests
import sickle
from confluent_kafka import Producer

from fatcat_tools.workers import most_recent_message
from .harvest_common import HarvestState


class HarvestOaiPmhWorker:
    """
    Base class for OAI-PMH harvesters. Uses the 'sickle' protocol library.

    Typically run as a single process; harvests records and publishes in raw
    (XML) format to a Kafka topic, one-message-per-document.

    Based on Crossref importer, with the HarvestState internal class managing
    progress with day-level granularity. Note that this depends on the OAI-PMH
    endpoint being correct! In that it must be possible to poll for only
    records updated on a particular date (typically "yesterday").

    Was very tempted to re-use <https://github.com/miku/metha> for this OAI-PMH
    stuff to save on dev time, but i'd already built the Crossref harvester and
    would want something similar operationally. Oh well!
    """


    def __init__(self, kafka_hosts, produce_topic, state_topic,
            start_date=None, end_date=None):

        self.produce_topic = produce_topic
        self.state_topic = state_topic
        self.kafka_config = {
            'bootstrap.servers': kafka_hosts,
            'delivery.report.only.error': True,
            'message.max.bytes': 20000000, # ~20 MBytes; broker is ~50 MBytes
            'default.topic.config':
                {'request.required.acks': 'all'},
        }

        self.loop_sleep = 60*60 # how long to wait, in seconds, between date checks

        self.endpoint_url = None # needs override
        self.metadata_prefix = None  # needs override
        self.name = "unnamed"
        self.state = HarvestState(start_date, end_date)
        self.state.initialize_from_kafka(self.state_topic, self.kafka_config)

    def fetch_date(self, date):

        def fail_fast(err, msg):
            if err is not None:
                print("Kafka producer delivery error: {}".format(err))
                print("Bailing out...")
                # TODO: should it be sys.exit(-1)?
                raise KafkaException(err)

        producer = Producer(self.kafka_config)

        api = sickle.Sickle(self.endpoint_url)
        date_str = date.isoformat()
        # this dict kwargs hack is to work around 'from' as a reserved python keyword
        # recommended by sickle docs
        try:
            records = api.ListRecords(**{
                'metadataPrefix': self.metadata_prefix,
                'from': date_str,
                'until': date_str,
            })
        except sickle.oaiexceptions.NoRecordsMatch:
            print("WARN: no OAI-PMH records for this date: {} (UTC)".format(date_str))
            return

        count = 0
        for item in records:
            count += 1
            if count % 50 == 0:
                print("... up to {}".format(count))
            producer.produce(
                self.produce_topic,
                item.raw.encode('utf-8'),
                key=item.header.identifier.encode('utf-8'),
                on_delivery=fail_fast)
            producer.poll(0)
        producer.flush()

    def run(self, continuous=False):

        while True:
            current = self.state.next(continuous)
            if current:
                print("Fetching DOIs updated on {} (UTC)".format(current))
                self.fetch_date(current)
                self.state.complete(current,
                    kafka_topic=self.state_topic,
                    kafka_config=self.kafka_config)
                continue

            if continuous:
                print("Sleeping {} seconds...".format(self.loop_sleep))
                time.sleep(self.loop_sleep)
            else:
                break
        print("{} OAI-PMH ingest caught up".format(self.name))


class HarvestArxivWorker(HarvestOaiPmhWorker):
    """
    Arxiv refs:
    - http://export.arxiv.org/oai2?verb=GetRecord&identifier=oai:arXiv.org:0804.2273&metadataPrefix=arXiv
    - http://export.arxiv.org/oai2?verb=GetRecord&identifier=oai:arXiv.org:0804.2273&metadataPrefix=arXivRaw

    All records are work-level. Some metadata formats have internal info about
    specific versions. The 'arXivRaw' format does, so i'm using that.
    """

    def __init__(self, **kwargs):
        super().__init__(**kwargs)
        self.endpoint_url = "https://export.arxiv.org/oai2"
        self.metadata_prefix = "arXivRaw"
        self.name = "arxiv"


class HarvestPubmedWorker(HarvestOaiPmhWorker):
    """
    Will likely be doing MEDLINE daily batch imports for primary metadata, but
    might also want to run a PMC importer to update fulltext and assign OA
    licenses (when appropriate).

    Pubmed refs:
    - https://www.ncbi.nlm.nih.gov/pmc/tools/oai/
    - https://www.ncbi.nlm.nih.gov/pmc/oai/oai.cgi?verb=GetRecord&identifier=oai:pubmedcentral.nih.gov:152494&metadataPrefix=pmc_fm
    - https://github.com/titipata/pubmed_parser
    """

    def __init__(self, **kwargs):
        super().__init__(**kwargs)
        self.endpoint_url = "https://www.ncbi.nlm.nih.gov/pmc/oai/oai.cgi"
        self.metadata_prefix = "pmc_fm"
        self.name = "pubmed"


class HarvestDoajJournalWorker(HarvestOaiPmhWorker):
    """
    WARNING: DOAJ OAI-PMH doesn't seem to respect 'from' and 'until' params

    As an alternative, could use:
    - https://github.com/miku/doajfetch
    """

    def __init__(self, **kwargs):
        super().__init__(**kwargs)
        self.endpoint_url = "https://www.doaj.org/oai"
        self.metadata_prefix = "oai_dc"
        self.name = "doaj-journal"


class HarvestDoajArticleWorker(HarvestOaiPmhWorker):
    """
    WARNING: DOAJ OAI-PMH doesn't seem to respect 'from' and 'until' params
    """

    def __init__(self, **kwargs):
        super().__init__(**kwargs)
        self.endpoint_url = "https://www.doaj.org/oai.article"
        self.metadata_prefix = "oai_doaj"
        self.name = "doaj-article"
