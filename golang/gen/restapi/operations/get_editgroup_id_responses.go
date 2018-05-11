// Code generated by go-swagger; DO NOT EDIT.

package operations

// This file was generated by the swagger tool.
// Editing this file might prove futile when you re-run the swagger generate command

import (
	"net/http"

	"github.com/go-openapi/runtime"

	models "git.archive.org/bnewbold/fatcat/golang/gen/models"
)

// GetEditgroupIDOKCode is the HTTP code returned for type GetEditgroupIDOK
const GetEditgroupIDOKCode int = 200

/*GetEditgroupIDOK fetch editgroup by identifier

swagger:response getEditgroupIdOK
*/
type GetEditgroupIDOK struct {

	/*
	  In: Body
	*/
	Payload *models.Editgroup `json:"body,omitempty"`
}

// NewGetEditgroupIDOK creates GetEditgroupIDOK with default headers values
func NewGetEditgroupIDOK() *GetEditgroupIDOK {

	return &GetEditgroupIDOK{}
}

// WithPayload adds the payload to the get editgroup Id o k response
func (o *GetEditgroupIDOK) WithPayload(payload *models.Editgroup) *GetEditgroupIDOK {
	o.Payload = payload
	return o
}

// SetPayload sets the payload to the get editgroup Id o k response
func (o *GetEditgroupIDOK) SetPayload(payload *models.Editgroup) {
	o.Payload = payload
}

// WriteResponse to the client
func (o *GetEditgroupIDOK) WriteResponse(rw http.ResponseWriter, producer runtime.Producer) {

	rw.WriteHeader(200)
	if o.Payload != nil {
		payload := o.Payload
		if err := producer.Produce(rw, payload); err != nil {
			panic(err) // let the recovery middleware deal with this
		}
	}
}

// GetEditgroupIDNotFoundCode is the HTTP code returned for type GetEditgroupIDNotFound
const GetEditgroupIDNotFoundCode int = 404

/*GetEditgroupIDNotFound no such editgroup

swagger:response getEditgroupIdNotFound
*/
type GetEditgroupIDNotFound struct {

	/*
	  In: Body
	*/
	Payload *models.Error `json:"body,omitempty"`
}

// NewGetEditgroupIDNotFound creates GetEditgroupIDNotFound with default headers values
func NewGetEditgroupIDNotFound() *GetEditgroupIDNotFound {

	return &GetEditgroupIDNotFound{}
}

// WithPayload adds the payload to the get editgroup Id not found response
func (o *GetEditgroupIDNotFound) WithPayload(payload *models.Error) *GetEditgroupIDNotFound {
	o.Payload = payload
	return o
}

// SetPayload sets the payload to the get editgroup Id not found response
func (o *GetEditgroupIDNotFound) SetPayload(payload *models.Error) {
	o.Payload = payload
}

// WriteResponse to the client
func (o *GetEditgroupIDNotFound) WriteResponse(rw http.ResponseWriter, producer runtime.Producer) {

	rw.WriteHeader(404)
	if o.Payload != nil {
		payload := o.Payload
		if err := producer.Produce(rw, payload); err != nil {
			panic(err) // let the recovery middleware deal with this
		}
	}
}

/*GetEditgroupIDDefault generic error response

swagger:response getEditgroupIdDefault
*/
type GetEditgroupIDDefault struct {
	_statusCode int

	/*
	  In: Body
	*/
	Payload *models.Error `json:"body,omitempty"`
}

// NewGetEditgroupIDDefault creates GetEditgroupIDDefault with default headers values
func NewGetEditgroupIDDefault(code int) *GetEditgroupIDDefault {
	if code <= 0 {
		code = 500
	}

	return &GetEditgroupIDDefault{
		_statusCode: code,
	}
}

// WithStatusCode adds the status to the get editgroup ID default response
func (o *GetEditgroupIDDefault) WithStatusCode(code int) *GetEditgroupIDDefault {
	o._statusCode = code
	return o
}

// SetStatusCode sets the status to the get editgroup ID default response
func (o *GetEditgroupIDDefault) SetStatusCode(code int) {
	o._statusCode = code
}

// WithPayload adds the payload to the get editgroup ID default response
func (o *GetEditgroupIDDefault) WithPayload(payload *models.Error) *GetEditgroupIDDefault {
	o.Payload = payload
	return o
}

// SetPayload sets the payload to the get editgroup ID default response
func (o *GetEditgroupIDDefault) SetPayload(payload *models.Error) {
	o.Payload = payload
}

// WriteResponse to the client
func (o *GetEditgroupIDDefault) WriteResponse(rw http.ResponseWriter, producer runtime.Producer) {

	rw.WriteHeader(o._statusCode)
	if o.Payload != nil {
		payload := o.Payload
		if err := producer.Produce(rw, payload); err != nil {
			panic(err) // let the recovery middleware deal with this
		}
	}
}
