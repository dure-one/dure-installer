package main

import (
	"testing"
)

func TestDetailedErrorCreation(t *testing.T) {
	err := NewDetailedError(
		ErrUserNotFound,
		"User 'testuser' not found",
		map[string]interface{}{"username": "testuser"},
	)

	if err.Code != ErrUserNotFound {
		t.Errorf("Expected code %d, got %d", ErrUserNotFound, err.Code)
	}
	if err.Type != "UserNotFound" {
		t.Errorf("Expected type 'UserNotFound', got '%s'", err.Type)
	}
	if err.Message != "User 'testuser' not found" {
		t.Errorf("Unexpected message: %s", err.Message)
	}
}

func TestDetailedErrorToErrorObj(t *testing.T) {
	err := NewDetailedError(
		ErrCredentialNotFound,
		"Credential not found",
		nil,
	)

	errObj := err.ToErrorObj()

	if errObj.Code != ErrCredentialNotFound {
		t.Errorf("Expected code %d, got %d", ErrCredentialNotFound, errObj.Code)
	}
	if errObj.Message != "Credential not found" {
		t.Errorf("Unexpected message: %s", errObj.Message)
	}
}

func TestErrorCodeTypes(t *testing.T) {
	tests := []struct {
		code     int
		expected string
	}{
		{ErrUserNotFound, "UserNotFound"},
		{ErrUserAlreadyExists, "UserAlreadyExists"},
		{ErrCredentialNotFound, "CredentialNotFound"},
		{ErrWebAuthnNotInitialized, "WebAuthnNotInitialized"},
		{ErrRateLimitExceeded, "RateLimitExceeded"},
	}

	for _, tt := range tests {
		err := NewDetailedError(tt.code, "test message", nil)
		if err.Type != tt.expected {
			t.Errorf("Code %d: expected type '%s', got '%s'", tt.code, tt.expected, err.Type)
		}
	}
}

func TestErrorWithSuggestion(t *testing.T) {
	err := NewDetailedError(
		ErrUserNotFound,
		"User not found",
		nil,
	)

	// Suggestion comes from metadata
	if err.Suggestion == "" {
		t.Error("Expected suggestion to be set from metadata")
	}

	// Verify it's the expected suggestion from metadata
	expectedSuggestion := "Check username or register first"
	if err.Suggestion != expectedSuggestion {
		t.Errorf("Expected suggestion '%s', got '%s'", expectedSuggestion, err.Suggestion)
	}
}

func TestErrorDetails(t *testing.T) {
	details := map[string]interface{}{
		"username":     "testuser",
		"attempted_at": "2026-06-26",
	}

	err := NewDetailedError(ErrUserNotFound, "User not found", details)

	// Verify details are stored in the DetailedError
	if err.Details == nil {
		t.Error("Expected details to be non-nil")
	}

	if err.Details["username"] != "testuser" {
		t.Error("Username not in error details")
	}

	if err.Details["attempted_at"] != "2026-06-26" {
		t.Error("attempted_at not in error details")
	}
}
