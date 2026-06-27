package main

// Error code constants for detailed error reporting
const (
	// User Errors (1000-1999)
	ErrUserNotFound       = 1001
	ErrUserAlreadyExists  = 1002
	ErrInvalidUsername    = 1003
	ErrUserDeleted        = 1004
	ErrTooManyUsers       = 1005

	// Credential Errors (2000-2999)
	ErrCredentialNotFound   = 2001
	ErrCredentialInvalid    = 2002
	ErrCredentialRevoked    = 2003
	ErrTooManyCredentials   = 2004
	ErrCredentialExpired    = 2005
	ErrDuplicateCredential  = 2006

	// WebAuthn Errors (3000-3999)
	ErrWebAuthnNotInitialized = 3001
	ErrInvalidChallenge       = 3002
	ErrVerificationFailed     = 3003
	ErrAttestationFailed      = 3004
	ErrInvalidSession         = 3005
	ErrSessionExpired         = 3006
	ErrInvalidOrigin          = 3007
	ErrInvalidRPID            = 3008

	// Crypto Errors (4000-4999)
	ErrInvalidKeyLength  = 4001
	ErrSignatureFailed   = 4002
	ErrDecryptionFailed  = 4003
	ErrEncryptionFailed  = 4004
	ErrInvalidSignature  = 4005
	ErrInvalidCiphertext = 4006

	// System Errors (5000-5999)
	ErrRateLimitExceeded = 5001
	ErrInternalError     = 5002
	ErrInvalidParameters = 5003
	ErrNotImplemented    = 5004
	ErrInvalidMethod     = 5005
)

// DetailedError provides rich error information
type DetailedError struct {
	Code       int                    `json:"code"`
	Type       string                 `json:"type"`
	Message    string                 `json:"message"`
	Details    map[string]interface{} `json:"details,omitempty"`
	Suggestion string                 `json:"suggestion,omitempty"`
}

// Error code metadata
var errorMetadata = map[int]struct {
	Type       string
	Suggestion string
}{
	ErrUserNotFound: {
		Type:       "UserNotFound",
		Suggestion: "Check username or register first",
	},
	ErrUserAlreadyExists: {
		Type:       "UserAlreadyExists",
		Suggestion: "Use a different username or login instead",
	},
	ErrInvalidUsername: {
		Type:       "InvalidUsername",
		Suggestion: "Username must be a valid email or identifier",
	},
	ErrUserDeleted: {
		Type:       "UserDeleted",
		Suggestion: "User account has been deleted",
	},
	ErrCredentialNotFound: {
		Type:       "CredentialNotFound",
		Suggestion: "Credential may have been removed or never registered",
	},
	ErrCredentialInvalid: {
		Type:       "CredentialInvalid",
		Suggestion: "Credential format is invalid",
	},
	ErrCredentialRevoked: {
		Type:       "CredentialRevoked",
		Suggestion: "This credential has been revoked",
	},
	ErrTooManyCredentials: {
		Type:       "TooManyCredentials",
		Suggestion: "Remove old credentials before adding new ones",
	},
	ErrWebAuthnNotInitialized: {
		Type:       "WebAuthnNotInitialized",
		Suggestion: "Call signup.begin first to initialize WebAuthn",
	},
	ErrInvalidChallenge: {
		Type:       "InvalidChallenge",
		Suggestion: "Challenge may have expired or been used",
	},
	ErrVerificationFailed: {
		Type:       "VerificationFailed",
		Suggestion: "Credential verification failed - check authenticator",
	},
	ErrAttestationFailed: {
		Type:       "AttestationFailed",
		Suggestion: "Attestation verification failed - device may not be trusted",
	},
	ErrInvalidSession: {
		Type:       "InvalidSession",
		Suggestion: "Session not found or invalid",
	},
	ErrSessionExpired: {
		Type:       "SessionExpired",
		Suggestion: "Session has expired - start new ceremony",
	},
	ErrInvalidKeyLength: {
		Type:       "InvalidKeyLength",
		Suggestion: "Key must be correct length (32 bytes for Ed25519 public, 64 for private)",
	},
	ErrSignatureFailed: {
		Type:       "SignatureFailed",
		Suggestion: "Failed to create signature",
	},
	ErrDecryptionFailed: {
		Type:       "DecryptionFailed",
		Suggestion: "Decryption failed - wrong key or corrupted data",
	},
	ErrEncryptionFailed: {
		Type:       "EncryptionFailed",
		Suggestion: "Encryption failed",
	},
	ErrRateLimitExceeded: {
		Type:       "RateLimitExceeded",
		Suggestion: "Too many requests - please wait before trying again",
	},
	ErrInternalError: {
		Type:       "InternalError",
		Suggestion: "Internal server error",
	},
	ErrInvalidParameters: {
		Type:       "InvalidParameters",
		Suggestion: "Check request parameters",
	},
	ErrNotImplemented: {
		Type:       "NotImplemented",
		Suggestion: "Feature not yet implemented",
	},
}

// NewDetailedError creates a detailed error with rich information
func NewDetailedError(code int, message string, details map[string]interface{}) *DetailedError {
	meta := errorMetadata[code]
	return &DetailedError{
		Code:       code,
		Type:       meta.Type,
		Message:    message,
		Details:    details,
		Suggestion: meta.Suggestion,
	}
}

// ToErrorObj converts DetailedError to JSON-RPC ErrorObj (for compatibility)
func (e *DetailedError) ToErrorObj() *ErrorObj {
	return &ErrorObj{
		Code:    e.Code,
		Message: e.Message,
	}
}
