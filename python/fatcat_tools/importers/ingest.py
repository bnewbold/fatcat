
import sys
import json
import base64
import itertools
import fatcat_openapi_client
from .common import EntityImporter, clean, make_rel_url, SANE_MAX_RELEASES, SANE_MAX_URLS, b32_hex


class IngestFileResultImporter(EntityImporter):

    def __init__(self, api, require_grobid=True, **kwargs):

        eg_desc = kwargs.pop('editgroup_description',
            "Files crawled from web using sandcrawler ingest tool")
        eg_extra = kwargs.pop('editgroup_extra', dict())
        eg_extra['agent'] = eg_extra.get('agent', 'fatcat_tools.IngestFileResultImporter')
        super().__init__(api,
            editgroup_description=eg_desc,
            editgroup_extra=eg_extra,
            **kwargs)
        self.default_link_rel = kwargs.get("default_link_rel", "web")
        assert self.default_link_rel
        self.default_mimetype = kwargs.get("default_mimetype", None)
        self.do_updates = kwargs.get("do_updates", False)
        self.require_grobid = require_grobid
        if self.require_grobid:
            print("Requiring GROBID status == 200")
        else:
            print("NOT checking GROBID success")
        self.ingest_request_source_whitelist = [
            'fatcat-changelog',
            'fatcat-ingest-container',
        ]
        if kwargs.get('skip_source_whitelist', False):
            self.ingest_request_source_whitelist = []

    def want(self, row):
        """
        Logic here probably needs work (TODO):

        - Direct ingests via DOI from fatcat-changelog should probably go
          through regardless of GROBID status
        - We should filter/block things like single-page PDFs here
        - public/anonymous submissions could require successful biblio-glutton
          match, or some other sanity check on the fatcat side (eg, fuzzy title
          match)
        - handle the case of release_stage not being 'published'; if pre-print,
          potentially create a new release.

        The current logic is intentionally conservative as a first step.
        """
        if row.get('hit') != True:
            self.counts['skip-hit'] += 1
            return False
        if self.ingest_request_source_whitelist and row['request'].get('ingest_request_source') not in self.ingest_request_source_whitelist:
            self.counts['skip-ingest_request_source'] += 1
            return False
        if not row.get('file_meta'):
            self.counts['skip-file-meta'] += 1
            return False
        if self.require_grobid and row.get('grobid', {}).get('status_code') != 200:
            self.counts['skip-grobid'] += 1
            return False

        return True

    def parse_record(self, row):

        request = row['request']
        fatcat = request.get('fatcat')
        file_meta = row['file_meta']

        # identify release by fatcat ident, or extid lookup, or biblio-glutton match
        release_ident = None
        if fatcat and fatcat.get('release_ident'):
            release_ident = fatcat.get('release_ident')
        elif request.get('ext_ids'):
            # if no fatcat ident, try extids
            for extid_type in ('doi', 'pmid', 'pmcid', 'arxiv'):
                extid = request['ext_ids'].get(extid_type)
                if not extid:
                    continue
                try:
                    release = self.api.lookup_release(**{extid_type: extid})
                except fatcat_openapi_client.rest.ApiException as err:
                    if err.status == 404:
                        continue
                    elif err.status == 400:
                        self.counts['warn-extid-invalid'] += 1
                        continue
                release_ident = release.ident
                break
        if not release_ident and row.get('grobid'):
            # try biblio-glutton extracted hit
            if row['grobid'].get('fatcat_ident'):
                release = row['grobid']['fatcat_ident'].split('_')[-1]

        if not release_ident:
            self.counts['skip-release-not-found'] += 1
            return None

        cdx = row.get('cdx')
        if not cdx:
            # TODO: support archive.org hits?
            self.counts['skip-no-cdx'] += 1
            return None

        url = make_rel_url(cdx['url'], self.default_link_rel)

        if not url:
            self.counts['skip-url'] += 1
            return None
        wayback = "https://web.archive.org/web/{}/{}".format(
            cdx['datetime'],
            cdx['url'])
        urls = [url, ("webarchive", wayback)]

        urls = [fatcat_openapi_client.FileUrl(rel=rel, url=url) for (rel, url) in urls]

        fe = fatcat_openapi_client.FileEntity(
            md5=file_meta['md5hex'],
            sha1=file_meta['sha1hex'],
            sha256=file_meta['sha256hex'],
            size=file_meta['size_bytes'],
            mimetype=file_meta['mimetype'] or self.default_mimetype,
            release_ids=[release_ident],
            urls=urls,
        )
        if fatcat and fatcat.get('edit_extra'):
            fe.edit_extra = fatcat['edit_extra']
        if request.get('ingest_request_source'):
            if not fe.edit_extra:
                fe.edit_extra = dict()
            fe.edit_extra['ingest_request_source'] = request['ingest_request_source']
        return fe

    def try_update(self, fe):
        # lookup sha1, or create new entity
        existing = None
        try:
            existing = self.api.lookup_file(sha1=fe.sha1)
        except fatcat_openapi_client.rest.ApiException as err:
            if err.status != 404:
                raise err

        if not existing:
            return True

        if (fe.release_ids[0] in existing.release_ids) and existing.urls:
            # TODO: could still, in theory update with the new URL?
            self.counts['exists'] += 1
            return False

        if not self.do_updates:
            self.counts['skip-update-disabled'] += 1
            return False

        # TODO: for now, never update
        self.counts['skip-update-disabled'] += 1
        return False

    def insert_batch(self, batch):
        self.api.create_file_auto_batch(fatcat_openapi_client.FileAutoBatch(
            editgroup=fatcat_openapi_client.Editgroup(
                description=self.editgroup_description,
                extra=self.editgroup_extra),
            entity_list=batch))

