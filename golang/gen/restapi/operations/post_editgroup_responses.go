// Code generated by go-swagger; DO NOT EDIT.

package operations

// This file was generated by the swagger tool.
// Editing this file might prove futile when you re-run the swagger generate command

import (
	"net/http"

	"github.com/go-openapi/runtime"

	models "git.archive.org/bnewbold/fatcat/golang/gen/models"
)

// PostEditgroupCreatedCode is the HTTP code returned for type PostEditgroupCreated
const PostEditgroupCreatedCode int = 201

/*PostEditgroupCreated successfully created

swagger:response postEditgroupCreated
*/
type PostEditgroupCreated struct {

	/*
	  In: Body
	*/
	Payload *models.Editgroup `json:"body,omitempty"`
}

// NewPostEditgroupCreated creates PostEditgroupCreated with default headers values
func NewPostEditgroupCreated() *PostEditgroupCreated {

	return &PostEditgroupCreated{}
}

// WithPayload adds the payload to the post editgroup created response
func (o *PostEditgroupCreated) WithPayload(payload *models.Editgroup) *PostEditgroupCreated {
	o.Payload = payload
	return o
}

// SetPayload sets the payload to the post editgroup created response
func (o *PostEditgroupCreated) SetPayload(payload *models.Editgroup) {
	o.Payload = payload
}

// WriteResponse to the client
func (o *PostEditgroupCreated) WriteResponse(rw http.ResponseWriter, producer runtime.Producer) {

	rw.WriteHeader(201)
	if o.Payload != nil {
		payload := o.Payload
		if err := producer.Produce(rw, payload); err != nil {
			panic(err) // let the recovery middleware deal with this
		}
	}
}

// PostEditgroupBadRequestCode is the HTTP code returned for type PostEditgroupBadRequest
const PostEditgroupBadRequestCode int = 400

/*PostEditgroupBadRequest invalid request parameters

swagger:response postEditgroupBadRequest
*/
type PostEditgroupBadRequest struct {

	/*
	  In: Body
	*/
	Payload *models.Error `json:"body,omitempty"`
}

// NewPostEditgroupBadRequest creates PostEditgroupBadRequest with default headers values
func NewPostEditgroupBadRequest() *PostEditgroupBadRequest {

	return &PostEditgroupBadRequest{}
}

// WithPayload adds the payload to the post editgroup bad request response
func (o *PostEditgroupBadRequest) WithPayload(payload *models.Error) *PostEditgroupBadRequest {
	o.Payload = payload
	return o
}

// SetPayload sets the payload to the post editgroup bad request response
func (o *PostEditgroupBadRequest) SetPayload(payload *models.Error) {
	o.Payload = payload
}

// WriteResponse to the client
func (o *PostEditgroupBadRequest) WriteResponse(rw http.ResponseWriter, producer runtime.Producer) {

	rw.WriteHeader(400)
	if o.Payload != nil {
		payload := o.Payload
		if err := producer.Produce(rw, payload); err != nil {
			panic(err) // let the recovery middleware deal with this
		}
	}
}

/*PostEditgroupDefault generic error response

swagger:response postEditgroupDefault
*/
type PostEditgroupDefault struct {
	_statusCode int

	/*
	  In: Body
	*/
	Payload *models.Error `json:"body,omitempty"`
}

// NewPostEditgroupDefault creates PostEditgroupDefault with default headers values
func NewPostEditgroupDefault(code int) *PostEditgroupDefault {
	if code <= 0 {
		code = 500
	}

	return &PostEditgroupDefault{
		_statusCode: code,
	}
}

// WithStatusCode adds the status to the post editgroup default response
func (o *PostEditgroupDefault) WithStatusCode(code int) *PostEditgroupDefault {
	o._statusCode = code
	return o
}

// SetStatusCode sets the status to the post editgroup default response
func (o *PostEditgroupDefault) SetStatusCode(code int) {
	o._statusCode = code
}

// WithPayload adds the payload to the post editgroup default response
func (o *PostEditgroupDefault) WithPayload(payload *models.Error) *PostEditgroupDefault {
	o.Payload = payload
	return o
}

// SetPayload sets the payload to the post editgroup default response
func (o *PostEditgroupDefault) SetPayload(payload *models.Error) {
	o.Payload = payload
}

// WriteResponse to the client
func (o *PostEditgroupDefault) WriteResponse(rw http.ResponseWriter, producer runtime.Producer) {

	rw.WriteHeader(o._statusCode)
	if o.Payload != nil {
		payload := o.Payload
		if err := producer.Produce(rw, payload); err != nil {
			panic(err) // let the recovery middleware deal with this
		}
	}
}
