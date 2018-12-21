
import json
import pytest
from copy import copy

from fatcat_client import *
from fatcat_client.rest import ApiException
from fixtures import *

def test_get_changelog_entry(api):
    """
    Basically just to check that fixture is working
    """
    cl = api.get_changelog_entry(1)
    assert cl

def quick_eg(api_inst):
    eg = api_inst.create_editgroup(
        fatcat_client.Editgroup(editor_id='aaaaaaaaaaaabkvkaaaaaaaaae'))
    return eg

def test_redirect_entity(api):
    """
    Create two creators; merge
        => get both by ident
        => lookup by orcid; should not get old/merged one
        => update first; check that get on second by ident returns updated record
        => split second back out and re-get by ident/orcid
    """

    offset = 0
    while True:
        offset += 1
        o1 = '0000-0000-1111-%04d' % offset
        o2 = '0000-0000-2222-%04d' % offset
        try:
            api.lookup_creator(orcid=o1)
            continue
        except ApiException:
            pass
        try:
            api.lookup_creator(orcid=o2)
            continue
        except ApiException:
            pass
        break

    c1 = CreatorEntity(display_name="test one", orcid=o1)
    c2 = CreatorEntity(display_name="test two", orcid=o2)

    # create two creators
    eg = quick_eg(api)
    c1 = api.get_creator(api.create_creator(c1, editgroup=eg.id).ident)
    c2 = api.get_creator(api.create_creator(c2, editgroup=eg.id).ident)
    api.accept_editgroup(eg.id)

    # merge second into first
    c2_redirect = CreatorEntity(redirect=c1.ident)
    eg = quick_eg(api)
    merge_edit = api.update_creator(c2.ident, c2_redirect, editgroup=eg.id)
    api.accept_editgroup(eg.id)

    # get both by ident
    res = api.get_creator(c1.ident)
    assert res.state == "active"
    res = api.get_creator(c2.ident)
    assert res.state == "redirect"
    assert res.revision == c1.revision
    assert res.redirect == c1.ident
    assert res.display_name == "test one"

    # get by orcid
    res = api.lookup_creator(orcid=o1)
    assert res.ident == c1.ident
    with pytest.raises(fatcat_client.rest.ApiException):
        res = api.lookup_creator(orcid=o2)

    # update first; check that get on second updates
    c1.display_name = "test one one"
    eg = quick_eg(api)
    api.update_creator(c1.ident, c1, editgroup=eg.id)
    api.accept_editgroup(eg.id)
    res = api.get_creator(c2.ident)
    assert res.state == "redirect"
    assert res.display_name == "test one one"

    # delete first; check that second is deleted (but state is redirect)
    eg = quick_eg(api)
    api.delete_creator(c1.ident, editgroup=eg.id)
    api.accept_editgroup(eg.id)
    res = api.get_creator(c1.ident)
    assert res.state == "deleted"
    assert res.display_name is None
    res = api.get_creator(c2.ident)
    assert res.state == "redirect"
    assert res.display_name is None
    assert res.revision is None

    # undelete first; check that second is a redirect
    eg = quick_eg(api)
    api.update_creator(c1.ident, c1, editgroup=eg.id)
    api.accept_editgroup(eg.id)
    res = api.get_creator(c2.ident)
    assert res.state == "redirect"
    assert res.display_name == "test one one"

    # split second entity back out
    assert c2.revision
    assert c2.redirect is None
    eg = quick_eg(api)
    api.update_creator(c2.ident, c2, editgroup=eg.id)
    api.accept_editgroup(eg.id)
    res = api.get_creator(c2.ident)
    assert res.state == "active"
    assert res.display_name == "test two"
    res = api.lookup_creator(orcid=o2)
    assert res.display_name == "test two"

    # cleanup
    eg = quick_eg(api)
    api.delete_creator(c1.ident)
    api.delete_creator(c2.ident)
    api.accept_editgroup(eg.id)

def test_delete_entity(api):

    offset = 0
    while True:
        offset += 1
        o1 = '0000-0000-1111-%04d' % offset
        try:
            api.lookup_creator(orcid=o1)
            continue
        except ApiException:
            pass
        break

    c1 = CreatorEntity(display_name="test deletable", orcid=o1)

    # create
    eg = quick_eg(api)
    c1 = api.get_creator(api.create_creator(c1, editgroup=eg.id).ident)
    api.accept_editgroup(eg.id)
    res = api.get_creator(c1.ident)
    assert res.state == "active"
    assert res.display_name == "test deletable"
    res = api.lookup_creator(orcid=c1.orcid)
    assert res.state == "active"
    assert res.display_name == "test deletable"

    # delete
    eg = quick_eg(api)
    api.delete_creator(c1.ident, editgroup=eg.id)
    api.accept_editgroup(eg.id)
    res = api.get_creator(c1.ident)
    assert res.state == "deleted"
    assert res.display_name is None
    with pytest.raises(fatcat_client.rest.ApiException):
        res = api.lookup_creator(orcid=c1.orcid)

    # undelete
    eg = quick_eg(api)
    api.update_creator(c1.ident, c1, editgroup=eg.id)
    api.accept_editgroup(eg.id)
    res = api.get_creator(c1.ident)
    assert res.state == "active"
    assert res.display_name == "test deletable"
    res = api.lookup_creator(orcid=c1.orcid)
    assert res.state == "active"
    assert res.display_name == "test deletable"

    # cleanup
    eg = quick_eg(api)
    api.delete_creator(c1.ident)
    api.accept_editgroup(eg.id)

def test_multiple_edits_same_group(api):

    c1 = CreatorEntity(display_name="test updates")

    # create
    eg = quick_eg(api)
    c1 = api.get_creator(api.create_creator(c1, editgroup=eg.id).ident)
    api.accept_editgroup(eg.id)

    # try multiple edits in the same group
    eg = quick_eg(api)
    c2 = CreatorEntity(display_name="left")
    c3 = CreatorEntity(display_name="right")
    edit = api.update_creator(c1.ident, c2, editgroup=eg.id)
    # should fail with existing
    with pytest.raises(fatcat_client.rest.ApiException):
        api.update_creator(c1.ident, c3, editgroup=eg.id)
    # ... but succeed after deleting
    api.delete_creator_edit(edit.edit_id)
    api.update_creator(c1.ident, c3, editgroup=eg.id)
    api.accept_editgroup(eg.id)
    res = api.get_creator(c1.ident)
    assert res.display_name == "right"
    eg = api.get_editgroup(eg.id)
    assert len(eg.edits.creators) == 1

    # cleanup
    eg = quick_eg(api)
    api.delete_creator(c1.ident)
    api.accept_editgroup(eg.id)

def test_edit_deletion(api):

    c1 = CreatorEntity(display_name="test edit updates")

    # create
    eg = quick_eg(api)
    c1 = api.get_creator(api.create_creator(c1, editgroup=eg.id).ident)
    api.accept_editgroup(eg.id)

    # try multiple edits in the same group
    c2 = CreatorEntity(display_name="update one")
    eg = quick_eg(api)
    eg = api.get_editgroup(eg.id)
    assert len(eg.edits.creators) == 0
    edit = api.update_creator(c1.ident, c2, editgroup=eg.id)
    eg = api.get_editgroup(eg.id)
    assert len(eg.edits.creators) == 1
    api.delete_creator_edit(edit.edit_id)
    eg = api.get_editgroup(eg.id)
    assert len(eg.edits.creators) == 0

    api.accept_editgroup(eg.id)
    res = api.get_creator(c1.ident)
    assert res.display_name == "test edit updates"
    eg = api.get_editgroup(eg.id)
    assert len(eg.edits.creators) == 0

    # cleanup
    eg = quick_eg(api)
    api.delete_creator(c1.ident)
    api.accept_editgroup(eg.id)

def test_empty_editgroup(api):
    eg = quick_eg(api)
    api.accept_editgroup(eg.id)

def test_recursive_redirects_entity(api):

    offset = 0
    while True:
        offset += 1
        o1 = '0000-0000-1111-%04d' % offset
        o2 = '0000-0000-2222-%04d' % offset
        o3 = '0000-0000-3333-%04d' % offset
        try:
            api.lookup_creator(orcid=o1)
            continue
        except ApiException:
            pass
        try:
            api.lookup_creator(orcid=o2)
            continue
        except ApiException:
            pass
        try:
            api.lookup_creator(orcid=o3)
            continue
        except ApiException:
            pass
        break

    c1 = CreatorEntity(display_name="test one", orcid=o1)
    c2 = CreatorEntity(display_name="test two", orcid=o2)
    c3 = CreatorEntity(display_name="test three", orcid=o3)

    # create three creators
    eg = quick_eg(api)
    c1 = api.get_creator(api.create_creator(c1, editgroup=eg.id).ident)
    c2 = api.get_creator(api.create_creator(c2, editgroup=eg.id).ident)
    c3 = api.get_creator(api.create_creator(c3, editgroup=eg.id).ident)
    api.accept_editgroup(eg.id)
    res = api.get_creator(c3.ident)
    assert res.display_name == "test three"

    # redirect third to second
    c3_redirect = CreatorEntity(redirect=c2.ident)
    eg = quick_eg(api)
    api.update_creator(c3.ident, c3_redirect, editgroup=eg.id)
    api.accept_editgroup(eg.id)
    res = api.get_creator(c3.ident)
    assert res.display_name == "test two"

    # redirect second to first: should be an error at merge time
    c2_redirect = CreatorEntity(redirect=c1.ident)
    eg = quick_eg(api)
    api.update_creator(c2.ident, c2_redirect, editgroup=eg.id)
    with pytest.raises(fatcat_client.rest.ApiException):
        api.accept_editgroup(eg.id)
    res = api.get_creator(c2.ident)
    assert res.display_name == "test two"

    # redirect first to third: should be an error at merge time
    c1_redirect = CreatorEntity(redirect=c3.ident)
    eg = quick_eg(api)
    api.update_creator(c1.ident, c1_redirect, editgroup=eg.id)
    with pytest.raises(fatcat_client.rest.ApiException):
        api.accept_editgroup(eg.id)
    res = api.get_creator(c1.ident)
    assert res.display_name == "test one"

    # update second; check that third updated
    c2.display_name = "test two updated"
    eg = quick_eg(api)
    api.update_creator(c2.ident, c2, editgroup=eg.id)
    api.accept_editgroup(eg.id)
    res = api.get_creator(c2.ident)
    c2 = res
    assert res.display_name == "test two updated"
    res = api.get_creator(c3.ident)
    assert res.display_name == "test two updated"
    assert res.state == "redirect"

    # delete second; check that third updated
    eg = quick_eg(api)
    api.delete_creator(c2.ident, editgroup=eg.id)
    api.accept_editgroup(eg.id)
    res = api.get_creator(c2.ident)
    assert res.state == "deleted"
    res = api.get_creator(c3.ident)
    assert res.state == "redirect"
    assert res.display_name is None

    # undelete second; check that third updated
    eg = quick_eg(api)
    c2_undelete = CreatorEntity(revision=c2.revision)
    api.update_creator(c2.ident, c2_undelete, editgroup=eg.id)
    api.accept_editgroup(eg.id)
    res = api.get_creator(c2.ident)
    assert res.state == "active"
    assert res.display_name == "test two updated"
    res = api.get_creator(c3.ident)
    assert res.state == "redirect"
    assert res.display_name == "test two updated"

    # delete third (a redirect)
    eg = quick_eg(api)
    api.delete_creator(c3.ident, editgroup=eg.id)
    api.accept_editgroup(eg.id)
    res = api.get_creator(c3.ident)
    assert res.state == "deleted"
    assert res.display_name is None

    # re-redirect third
    eg = quick_eg(api)
    api.update_creator(c3.ident, c3_redirect, editgroup=eg.id)
    api.accept_editgroup(eg.id)
    res = api.get_creator(c3.ident)
    assert res.state == "redirect"
    assert res.display_name == "test two updated"

    # delete second, then delete third
    eg = quick_eg(api)
    api.delete_creator(c2.ident, editgroup=eg.id)
    api.accept_editgroup(eg.id)
    res = api.get_creator(c3.ident)
    assert res.state == "redirect"
    assert res.display_name is None
    eg = quick_eg(api)
    api.delete_creator(c3.ident, editgroup=eg.id)
    api.accept_editgroup(eg.id)
    res = api.get_creator(c3.ident)
    assert res.state == "deleted"
    assert res.display_name is None

    # cleanup
    eg = quick_eg(api)
    api.delete_creator(c1.ident)
    # c2 already deleted
    # c3 already deleted
    api.accept_editgroup(eg.id)

