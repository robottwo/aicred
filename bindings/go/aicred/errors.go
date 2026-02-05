package aicred

import "errors"

// Common error definitions for aicred library
var (
	// ErrInstanceNotFound is returned when a provider instance is not found
	ErrInstanceNotFound = errors.New("provider instance not found")

	// ErrModelNotFound is returned when a model is not found
	ErrModelNotFound = errors.New("model not found")

	// ErrTagNotFound is returned when a tag is not found
	ErrTagNotFound = errors.New("tag not found")

	// ErrLabelNotFound is returned when a label is not found
	ErrLabelNotFound = errors.New("label not found")

	// ErrInvalidConfig is returned when configuration is invalid
	ErrInvalidConfig = errors.New("invalid configuration")

	// ErrValidationFailed is returned when validation fails
	ErrValidationFailed = errors.New("validation failed")

	// ErrLabelAlreadyAssigned is returned when a label is already assigned to a different target
	ErrLabelAlreadyAssigned = errors.New("label already assigned to a different target")

	// ErrInvalidTarget is returned when a target is invalid
	ErrInvalidTarget = errors.New("invalid target")
)

// Error is the interface for all aicred errors
type Error struct {
	// The underlying error
	Err error
	// Contextual information about the error
	Message string
	// The field that caused the error, if applicable
	Field string
}

// Error implements the error interface
func (e *Error) Error() string {
	if e.Message != "" {
		if e.Field != "" {
			return e.Message + ": " + e.Field
		}
		return e.Message
	}
	if e.Err != nil {
		return e.Err.Error()
	}
	return "unknown error"
}

// Unwrap returns the underlying error
func (e *Error) Unwrap() error {
	return e.Err
}

// NewError creates a new Error with a message
func NewError(message string) *Error {
	return &Error{Message: message}
}

// WrapError wraps an existing error with additional context
func WrapError(err error, message string) *Error {
	return &Error{Err: err, Message: message}
}

// ValidationError represents a validation error with field information
type ValidationError struct {
	Message string
	Field   string
	Value   interface{}
}

// Error implements the error interface
func (ve *ValidationError) Error() string {
	msg := ve.Message
	if ve.Field != "" {
		msg += " (field: " + ve.Field + ")"
	}
	return msg
}

// NewValidationError creates a new ValidationError
func NewValidationError(message, field string) *ValidationError {
	return &ValidationError{
		Message: message,
		Field:   field,
	}
}

// ValidationErrorf creates a new ValidationError with formatted message
func ValidationErrorf(format string, args ...interface{}) *ValidationError {
	return &ValidationError{
		Message: format,
	}
}
