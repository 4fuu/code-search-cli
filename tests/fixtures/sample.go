package sample

import (
	"context"
	"fmt"
	"sync"
)

const MaxRetries = 5

var DefaultTimeout = 30

type Result struct {
	Value interface{}
	Error error
}

type Handler interface {
	Handle(ctx context.Context, req Request) (*Response, error)
	Name() string
}

type Request struct {
	Method string
	Path   string
	Body   []byte
}

type Response struct {
	Status int
	Body   []byte
}

type Server struct {
	mu       sync.RWMutex
	handlers map[string]Handler
	port     int
}

func NewServer(port int) *Server {
	return &Server{
		handlers: make(map[string]Handler),
		port:     port,
	}
}

func (s *Server) Register(path string, handler Handler) {
	s.mu.Lock()
	defer s.mu.Unlock()
	s.handlers[path] = handler
}

func (s *Server) Start(ctx context.Context) error {
	fmt.Printf("Starting server on port %d\n", s.port)
	return nil
}

func (s *Server) handleRequest(ctx context.Context, req Request) (*Response, error) {
	s.mu.RLock()
	handler, ok := s.handlers[req.Path]
	s.mu.RUnlock()

	if !ok {
		return &Response{Status: 404}, nil
	}

	return handler.Handle(ctx, req)
}

type EchoHandler struct{}

func (h *EchoHandler) Handle(ctx context.Context, req Request) (*Response, error) {
	return &Response{
		Status: 200,
		Body:   req.Body,
	}, nil
}

func (h *EchoHandler) Name() string {
	return "echo"
}
