// Code generated by go-swagger; DO NOT EDIT.

package operations

// This file was generated by the swagger tool.
// Editing this file might prove futile when you re-run the swagger generate command

import (
	"net/http"

	"github.com/go-openapi/runtime"

	models "git.archive.org/bnewbold/fatcat/golang/gen/models"
)

// PostEditgroupIDAcceptOKCode is the HTTP code returned for type PostEditgroupIDAcceptOK
const PostEditgroupIDAcceptOKCode int = 200

/*PostEditgroupIDAcceptOK merged editgroup successfully ("live")

swagger:response postEditgroupIdAcceptOK
*/
type PostEditgroupIDAcceptOK struct {

	/*
	  In: Body
	*/
	Payload *models.Success `json:"body,omitempty"`
}

// NewPostEditgroupIDAcceptOK creates PostEditgroupIDAcceptOK with default headers values
func NewPostEditgroupIDAcceptOK() *PostEditgroupIDAcceptOK {

	return &PostEditgroupIDAcceptOK{}
}

// WithPayload adds the payload to the post editgroup Id accept o k response
func (o *PostEditgroupIDAcceptOK) WithPayload(payload *models.Success) *PostEditgroupIDAcceptOK {
	o.Payload = payload
	return o
}

// SetPayload sets the payload to the post editgroup Id accept o k response
func (o *PostEditgroupIDAcceptOK) SetPayload(payload *models.Success) {
	o.Payload = payload
}

// WriteResponse to the client
func (o *PostEditgroupIDAcceptOK) WriteResponse(rw http.ResponseWriter, producer runtime.Producer) {

	rw.WriteHeader(200)
	if o.Payload != nil {
		payload := o.Payload
		if err := producer.Produce(rw, payload); err != nil {
			panic(err) // let the recovery middleware deal with this
		}
	}
}

// PostEditgroupIDAcceptBadRequestCode is the HTTP code returned for type PostEditgroupIDAcceptBadRequest
const PostEditgroupIDAcceptBadRequestCode int = 400

/*PostEditgroupIDAcceptBadRequest editgroup is in an unmergable state

swagger:response postEditgroupIdAcceptBadRequest
*/
type PostEditgroupIDAcceptBadRequest struct {

	/*
	  In: Body
	*/
	Payload *models.Error `json:"body,omitempty"`
}

// NewPostEditgroupIDAcceptBadRequest creates PostEditgroupIDAcceptBadRequest with default headers values
func NewPostEditgroupIDAcceptBadRequest() *PostEditgroupIDAcceptBadRequest {

	return &PostEditgroupIDAcceptBadRequest{}
}

// WithPayload adds the payload to the post editgroup Id accept bad request response
func (o *PostEditgroupIDAcceptBadRequest) WithPayload(payload *models.Error) *PostEditgroupIDAcceptBadRequest {
	o.Payload = payload
	return o
}

// SetPayload sets the payload to the post editgroup Id accept bad request response
func (o *PostEditgroupIDAcceptBadRequest) SetPayload(payload *models.Error) {
	o.Payload = payload
}

// WriteResponse to the client
func (o *PostEditgroupIDAcceptBadRequest) WriteResponse(rw http.ResponseWriter, producer runtime.Producer) {

	rw.WriteHeader(400)
	if o.Payload != nil {
		payload := o.Payload
		if err := producer.Produce(rw, payload); err != nil {
			panic(err) // let the recovery middleware deal with this
		}
	}
}

// PostEditgroupIDAcceptNotFoundCode is the HTTP code returned for type PostEditgroupIDAcceptNotFound
const PostEditgroupIDAcceptNotFoundCode int = 404

/*PostEditgroupIDAcceptNotFound no such editgroup

swagger:response postEditgroupIdAcceptNotFound
*/
type PostEditgroupIDAcceptNotFound struct {

	/*
	  In: Body
	*/
	Payload *models.Error `json:"body,omitempty"`
}

// NewPostEditgroupIDAcceptNotFound creates PostEditgroupIDAcceptNotFound with default headers values
func NewPostEditgroupIDAcceptNotFound() *PostEditgroupIDAcceptNotFound {

	return &PostEditgroupIDAcceptNotFound{}
}

// WithPayload adds the payload to the post editgroup Id accept not found response
func (o *PostEditgroupIDAcceptNotFound) WithPayload(payload *models.Error) *PostEditgroupIDAcceptNotFound {
	o.Payload = payload
	return o
}

// SetPayload sets the payload to the post editgroup Id accept not found response
func (o *PostEditgroupIDAcceptNotFound) SetPayload(payload *models.Error) {
	o.Payload = payload
}

// WriteResponse to the client
func (o *PostEditgroupIDAcceptNotFound) WriteResponse(rw http.ResponseWriter, producer runtime.Producer) {

	rw.WriteHeader(404)
	if o.Payload != nil {
		payload := o.Payload
		if err := producer.Produce(rw, payload); err != nil {
			panic(err) // let the recovery middleware deal with this
		}
	}
}

/*PostEditgroupIDAcceptDefault generic error response

swagger:response postEditgroupIdAcceptDefault
*/
type PostEditgroupIDAcceptDefault struct {
	_statusCode int

	/*
	  In: Body
	*/
	Payload *models.Error `json:"body,omitempty"`
}

// NewPostEditgroupIDAcceptDefault creates PostEditgroupIDAcceptDefault with default headers values
func NewPostEditgroupIDAcceptDefault(code int) *PostEditgroupIDAcceptDefault {
	if code <= 0 {
		code = 500
	}

	return &PostEditgroupIDAcceptDefault{
		_statusCode: code,
	}
}

// WithStatusCode adds the status to the post editgroup ID accept default response
func (o *PostEditgroupIDAcceptDefault) WithStatusCode(code int) *PostEditgroupIDAcceptDefault {
	o._statusCode = code
	return o
}

// SetStatusCode sets the status to the post editgroup ID accept default response
func (o *PostEditgroupIDAcceptDefault) SetStatusCode(code int) {
	o._statusCode = code
}

// WithPayload adds the payload to the post editgroup ID accept default response
func (o *PostEditgroupIDAcceptDefault) WithPayload(payload *models.Error) *PostEditgroupIDAcceptDefault {
	o.Payload = payload
	return o
}

// SetPayload sets the payload to the post editgroup ID accept default response
func (o *PostEditgroupIDAcceptDefault) SetPayload(payload *models.Error) {
	o.Payload = payload
}

// WriteResponse to the client
func (o *PostEditgroupIDAcceptDefault) WriteResponse(rw http.ResponseWriter, producer runtime.Producer) {

	rw.WriteHeader(o._statusCode)
	if o.Payload != nil {
		payload := o.Payload
		if err := producer.Produce(rw, payload); err != nil {
			panic(err) // let the recovery middleware deal with this
		}
	}
}
