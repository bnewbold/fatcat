// Code generated by go-swagger; DO NOT EDIT.

package operations

// This file was generated by the swagger tool.
// Editing this file might prove futile when you re-run the swagger generate command

import (
	"net/http"

	"github.com/go-openapi/errors"
	"github.com/go-openapi/runtime"
	"github.com/go-openapi/runtime/middleware"
	"github.com/go-openapi/validate"

	strfmt "github.com/go-openapi/strfmt"
)

// NewGetCreatorLookupParams creates a new GetCreatorLookupParams object
// no default values defined in spec.
func NewGetCreatorLookupParams() GetCreatorLookupParams {

	return GetCreatorLookupParams{}
}

// GetCreatorLookupParams contains all the bound params for the get creator lookup operation
// typically these are obtained from a http.Request
//
// swagger:parameters GetCreatorLookup
type GetCreatorLookupParams struct {

	// HTTP Request Object
	HTTPRequest *http.Request `json:"-"`

	/*
	  Required: true
	  In: query
	*/
	Orcid string
}

// BindRequest both binds and validates a request, it assumes that complex things implement a Validatable(strfmt.Registry) error interface
// for simple values it will use straight method calls.
//
// To ensure default values, the struct must have been initialized with NewGetCreatorLookupParams() beforehand.
func (o *GetCreatorLookupParams) BindRequest(r *http.Request, route *middleware.MatchedRoute) error {
	var res []error

	o.HTTPRequest = r

	qs := runtime.Values(r.URL.Query())

	qOrcid, qhkOrcid, _ := qs.GetOK("orcid")
	if err := o.bindOrcid(qOrcid, qhkOrcid, route.Formats); err != nil {
		res = append(res, err)
	}

	if len(res) > 0 {
		return errors.CompositeValidationError(res...)
	}
	return nil
}

func (o *GetCreatorLookupParams) bindOrcid(rawData []string, hasKey bool, formats strfmt.Registry) error {
	if !hasKey {
		return errors.Required("orcid", "query")
	}
	var raw string
	if len(rawData) > 0 {
		raw = rawData[len(rawData)-1]
	}

	// Required: true
	// AllowEmptyValue: false
	if err := validate.RequiredString("orcid", "query", raw); err != nil {
		return err
	}

	o.Orcid = raw

	return nil
}
