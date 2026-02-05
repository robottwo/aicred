package aicred

import (
	"errors"
	"testing"
)

func TestErrorDefinitions(t *testing.T) {
	tests := []struct {
		name  string
		error error
	}{
		{"ErrInstanceNotFound", ErrInstanceNotFound},
		{"ErrModelNotFound", ErrModelNotFound},
		{"ErrTagNotFound", ErrTagNotFound},
		{"ErrLabelNotFound", ErrLabelNotFound},
		{"ErrInvalidConfig", ErrInvalidConfig},
		{"ErrValidationFailed", ErrValidationFailed},
		{"ErrLabelAlreadyAssigned", ErrLabelAlreadyAssigned},
		{"ErrInvalidTarget", ErrInvalidTarget},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			if tt.error == nil {
				t.Errorf("Error %s should not be nil", tt.name)
			}
			if tt.error.Error() == "" {
				t.Errorf("Error %s should have a message", tt.name)
			}
		})
	}
}

func TestNewError(t *testing.T) {
	err := NewError("test error message")
	if err.Message != "test error message" {
		t.Errorf("Expected message 'test error message', got %s", err.Message)
	}

	if err.Error() != "test error message" {
		t.Errorf("Expected Error() to return 'test error message', got %s", err.Error())
	}
}

func TestWrapError(t *testing.T) {
	baseErr := errors.New("base error")
	wrapped := WrapError(baseErr, "wrapped message")

	if wrapped.Err != baseErr {
		t.Error("Underlying error not set correctly")
	}

	if wrapped.Message != "wrapped message" {
		t.Errorf("Expected message 'wrapped message', got %s", wrapped.Message)
	}

	if wrapped.Error() != "wrapped message" {
		t.Errorf("Expected Error() to return 'wrapped message', got %s", wrapped.Error())
	}
}

func TestErrorUnwrap(t *testing.T) {
	baseErr := errors.New("base error")
	wrapped := WrapError(baseErr, "wrapped message")

	unwrapped := wrapped.Unwrap()
	if unwrapped != baseErr {
		t.Errorf("Unwrap() should return base error, got %v", unwrapped)
	}
}

func TestErrorWithField(t *testing.T) {
	err := &Error{
		Message: "test error",
		Field:   "test_field",
	}

	expected := "test error: test_field"
	if err.Error() != expected {
		t.Errorf("Expected '%s', got %s", expected, err.Error())
	}
}

func TestValidationError(t *testing.T) {
	err := NewValidationError("field is required", "field_name")

	if err.Message != "field is required" {
		t.Errorf("Expected message 'field is required', got %s", err.Message)
	}

	if err.Field != "field_name" {
		t.Errorf("Expected field 'field_name', got %s", err.Field)
	}

	expected := "field is required (field: field_name)"
	if err.Error() != expected {
		t.Errorf("Expected '%s', got %s", expected, err.Error())
	}
}

func TestValidationErrorWithoutField(t *testing.T) {
	err := NewValidationError("generic error", "")

	expected := "generic error"
	if err.Error() != expected {
		t.Errorf("Expected '%s', got %s", expected, err.Error())
	}
}

func TestValidationErrorf(t *testing.T) {
	err := ValidationErrorf("invalid value: %s", "test")

	if err.Message != "invalid value: %s" {
		t.Errorf("Expected message to be preserved, got %s", err.Message)
	}

	if err.Error() != "invalid value: %s" {
		t.Errorf("Expected message to be returned, got %s", err.Error())
	}
}

func TestErrorNilUnderlying(t *testing.T) {
	err := &Error{
		Message: "test error",
		Err:     nil,
	}

	// Should not panic
	_ = err.Unwrap()
	_ = err.Error()
}

func TestErrorEmptyMessage(t *testing.T) {
	err := &Error{
		Message: "",
		Err:     errors.New("base error"),
	}

	// Should return underlying error message
	expected := "base error"
	if err.Error() != expected {
		t.Errorf("Expected '%s', got %s", expected, err.Error())
	}
}

func TestErrorEmptyMessageAndNilErr(t *testing.T) {
	err := &Error{
		Message: "",
		Err:     nil,
	}

	// Should return default message
	expected := "unknown error"
	if err.Error() != expected {
		t.Errorf("Expected '%s', got %s", expected, err.Error())
	}
}
