// Code generated by go-swagger; DO NOT EDIT.

package operations

// This file was generated by the swagger tool.
// Editing this file might prove futile when you re-run the generate command

import (
	"net/http"

	middleware "github.com/go-openapi/runtime/middleware"
)

// PostEditgroupIDAcceptHandlerFunc turns a function with the right signature into a post editgroup ID accept handler
type PostEditgroupIDAcceptHandlerFunc func(PostEditgroupIDAcceptParams) middleware.Responder

// Handle executing the request and returning a response
func (fn PostEditgroupIDAcceptHandlerFunc) Handle(params PostEditgroupIDAcceptParams) middleware.Responder {
	return fn(params)
}

// PostEditgroupIDAcceptHandler interface for that can handle valid post editgroup ID accept params
type PostEditgroupIDAcceptHandler interface {
	Handle(PostEditgroupIDAcceptParams) middleware.Responder
}

// NewPostEditgroupIDAccept creates a new http.Handler for the post editgroup ID accept operation
func NewPostEditgroupIDAccept(ctx *middleware.Context, handler PostEditgroupIDAcceptHandler) *PostEditgroupIDAccept {
	return &PostEditgroupIDAccept{Context: ctx, Handler: handler}
}

/*PostEditgroupIDAccept swagger:route POST /editgroup/{id}/accept postEditgroupIdAccept

PostEditgroupIDAccept post editgroup ID accept API

*/
type PostEditgroupIDAccept struct {
	Context *middleware.Context
	Handler PostEditgroupIDAcceptHandler
}

func (o *PostEditgroupIDAccept) ServeHTTP(rw http.ResponseWriter, r *http.Request) {
	route, rCtx, _ := o.Context.RouteInfo(r)
	if rCtx != nil {
		r = rCtx
	}
	var Params = NewPostEditgroupIDAcceptParams()

	if err := o.Context.BindValidRequest(r, route, &Params); err != nil { // bind params
		o.Context.Respond(rw, r, route.Produces, route, err)
		return
	}

	res := o.Handler.Handle(Params) // actually handle the request

	o.Context.Respond(rw, r, route.Produces, route, res)

}
