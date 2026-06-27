package main

import (
	"bufio"
	"crypto/ed25519"
	"crypto/rand"
	"encoding/base64"
	"encoding/json"
	"flag"
	"fmt"
	"os"
	"runtime"
	"sync"
	"time"

	"github.com/go-webauthn/webauthn/protocol"
	"github.com/go-webauthn/webauthn/webauthn"
	"github.com/google/uuid"
	"golang.org/x/crypto/chacha20poly1305"
)

// JSON-RPC 2.0 structures
type Request struct {
	ID     string          `json:"id"`
	Method string          `json:"method"`
	Params json.RawMessage `json:"params,omitempty"`
}

type Response struct {
	ID     string      `json:"id"`
	Result interface{} `json:"result,omitempty"`
	Error  *ErrorObj   `json:"error,omitempty"`
}

type ErrorObj struct {
	Code    int    `json:"code"`
	Message string `json:"message"`
}

// Ed25519 request/response types
type Ed25519GenerateKeyResult struct {
	PublicKey  []byte `json:"public_key"`
	PrivateKey []byte `json:"private_key"`
}

type Ed25519SignParams struct {
	PrivateKey []byte `json:"private_key"`
	Message    []byte `json:"message"`
}

type Ed25519SignResult struct {
	Signature []byte `json:"signature"`
}

type Ed25519VerifyParams struct {
	PublicKey []byte `json:"public_key"`
	Message   []byte `json:"message"`
	Signature []byte `json:"signature"`
}

type Ed25519VerifyResult struct {
	Valid bool `json:"valid"`
}

// ChaCha20-Poly1305 request/response types
type ChaCha20EncryptParams struct {
	Key            []byte `json:"key"`
	Nonce          []byte `json:"nonce"`
	Plaintext      []byte `json:"plaintext"`
	AdditionalData []byte `json:"additional_data,omitempty"`
}

type ChaCha20EncryptResult struct {
	Ciphertext []byte `json:"ciphertext"`
}

type ChaCha20DecryptParams struct {
	Key            []byte `json:"key"`
	Nonce          []byte `json:"nonce"`
	Ciphertext     []byte `json:"ciphertext"`
	AdditionalData []byte `json:"additional_data,omitempty"`
}

type ChaCha20DecryptResult struct {
	Plaintext []byte `json:"plaintext"`
}

// WebAuthn request/response types
type WebAuthnSignupBeginParams struct {
	RPDisplayName string `json:"rp_display_name"`
	RPID          string `json:"rp_id"`
	RPOrigins     string `json:"rp_origins"` // Comma-separated
	Username      string `json:"username"`
	DisplayName   string `json:"display_name"`
	Scenario      string `json:"scenario"` // "mfa", "passwordless", or "usernameless"
}

type WebAuthnSignupBeginResult struct {
	SessionID     string `json:"session_id"`
	ChallengeJSON string `json:"challenge_json"`
}

type WebAuthnSignupFinishParams struct {
	SessionID      string `json:"session_id"`
	CredentialJSON string `json:"credential_json"`
}

type WebAuthnSignupFinishResult struct {
	UserID       string `json:"user_id"`
	CredentialID string `json:"credential_id"`
}

type WebAuthnSigninBeginParams struct {
	Username string `json:"username"`
	Scenario string `json:"scenario"`
}

type WebAuthnSigninBeginResult struct {
	SessionID     string `json:"session_id"`
	ChallengeJSON string `json:"challenge_json"`
}

type WebAuthnSigninFinishParams struct {
	SessionID      string `json:"session_id"`
	CredentialJSON string `json:"credential_json"`
}

type WebAuthnSigninFinishResult struct {
	UserID   string `json:"user_id"`
	Username string `json:"username"`
}

// Passkey Login (Discoverable Credentials) types
type WebAuthnPasskeyLoginBeginParams struct {
	RPDisplayName string `json:"rp_display_name"`
	RPID          string `json:"rp_id"`
	RPOrigins     string `json:"rp_origins"`
	Mediation     string `json:"mediation"` // "silent", "optional", "conditional", "required"
}

type WebAuthnPasskeyLoginBeginResult struct {
	SessionID     string `json:"session_id"`
	ChallengeJSON string `json:"challenge_json"`
}

type WebAuthnPasskeyLoginFinishParams struct {
	SessionID      string `json:"session_id"`
	CredentialJSON string `json:"credential_json"`
}

type WebAuthnPasskeyLoginFinishResult struct {
	UserID   string `json:"user_id"`
	Username string `json:"username"`
}

// MFA Login types
type WebAuthnMfaLoginBeginParams struct {
	Username  string `json:"username"`
	Mediation string `json:"mediation"`
}

type WebAuthnMfaLoginBeginResult struct {
	SessionID     string `json:"session_id"`
	ChallengeJSON string `json:"challenge_json"`
}

type WebAuthnMfaLoginFinishParams struct {
	SessionID      string `json:"session_id"`
	CredentialJSON string `json:"credential_json"`
}

type WebAuthnMfaLoginFinishResult struct {
	UserID   string `json:"user_id"`
	Username string `json:"username"`
	Verified bool   `json:"verified"`
}

// Credential Management types
type WebAuthnListCredentialsParams struct {
	Username string `json:"username"`
}

type WebAuthnCredentialInfo struct {
	ID              string `json:"id"`
	PublicKey       string `json:"public_key"` // base64
	SignCount       uint32 `json:"sign_count"`
	AAGUID          string `json:"aaguid"`
	AttestationType string `json:"attestation_type"`
}

type WebAuthnListCredentialsResult struct {
	Credentials []WebAuthnCredentialInfo `json:"credentials"`
}

type WebAuthnDeleteCredentialParams struct {
	Username     string `json:"username"`
	CredentialID string `json:"credential_id"`
}

type WebAuthnDeleteCredentialResult struct {
	Success bool `json:"success"`
}

// Global state for WebAuthn
var (
	webAuthnState     *WebAuthnState
	webAuthnStateLock sync.Mutex
)

func main() {
	// Parse command-line flags
	debugMode := flag.Bool("debug", false, "Enable debug logging")
	flag.Parse()

	if *debugMode {
		EnableDebugMode()
	}

	// Log startup
	Info("go-webauthn JSON-RPC server starting", map[string]interface{}{
		"version":    Version,
		"go_version": runtime.Version(),
		"debug_mode": *debugMode,
	})

	// Start background cleanup goroutine
	go func() {
		ticker := time.NewTicker(5 * time.Minute)
		defer ticker.Stop()
		for range ticker.C {
			CleanupRateLimiters()
		}
	}()

	scanner := bufio.NewScanner(os.Stdin)
	encoder := json.NewEncoder(os.Stdout)

	for scanner.Scan() {
		line := scanner.Bytes()

		var req Request
		if err := json.Unmarshal(line, &req); err != nil {
			LogError("Failed to parse request", map[string]interface{}{
				"error": err.Error(),
			})
			continue
		}

		// Handle request with logging and metrics
		resp := handleRequestWithInstrumentation(&req)

		if err := encoder.Encode(resp); err != nil {
			LogError("Failed to encode response", map[string]interface{}{
				"error":      err.Error(),
				"request_id": req.ID,
			})
		}
	}

	if err := scanner.Err(); err != nil {
		LogError("Scanner error", map[string]interface{}{
			"error": err.Error(),
		})
		os.Exit(1)
	}
}

// handleRequestWithInstrumentation wraps handleRequest with logging, metrics, and rate limiting
func handleRequestWithInstrumentation(req *Request) *Response {
	start := time.Now()

	// Log request
	LogRequest(req.Method, req.ID)

	// Extract username for rate limiting
	username := extractUsername(req)

	// Check rate limit (skip for system endpoints)
	if !isSystemEndpoint(req.Method) && !CheckRateLimit(username, req.Method) {
		LogError("Rate limit exceeded", map[string]interface{}{
			"username": username,
			"method":   req.Method,
		})
		return &Response{
			ID: req.ID,
			Error: &ErrorObj{
				Code:    ErrRateLimitExceeded,
				Message: "Rate limit exceeded - too many requests",
			},
		}
	}

	// Handle request
	resp := handleRequest(req)

	// Record metrics
	duration := time.Since(start)
	success := resp.Error == nil
	RecordRequest(req.Method, duration, success)
	LogResponse(req.Method, req.ID, duration, success)

	return resp
}

// isSystemEndpoint checks if a method is a system endpoint (exempt from rate limiting)
func isSystemEndpoint(method string) bool {
	return method == "health" || method == "metrics" || method == "version"
}

// extractUsername extracts username from request params
func extractUsername(req *Request) string {
	var params map[string]interface{}
	if err := json.Unmarshal(req.Params, &params); err != nil {
		return "anonymous"
	}
	if username, ok := params["username"].(string); ok {
		return username
	}
	return "anonymous"
}

// WebAuthn state management
type WebAuthnState struct {
	webauthn     *webauthn.WebAuthn
	users        map[string]*WebAuthnUser
	nameToID     map[string]string
	credentials  map[string][]webauthn.Credential
	regSessions  map[string]*RegistrationSession
	authSessions map[string]*AuthenticationSession
	mu           sync.RWMutex
}

type WebAuthnUser struct {
	id          []byte
	name        string
	displayName string
}

func (u *WebAuthnUser) WebAuthnID() []byte                { return u.id }
func (u *WebAuthnUser) WebAuthnName() string              { return u.name }
func (u *WebAuthnUser) WebAuthnDisplayName() string       { return u.displayName }
func (u *WebAuthnUser) WebAuthnIcon() string              { return "" }
func (u *WebAuthnUser) WebAuthnCredentials() []webauthn.Credential {
	return nil // Will be looked up from state
}

type RegistrationSession struct {
	UserID      string
	Username    string
	SessionData *webauthn.SessionData
}

type AuthenticationSession struct {
	UserID      string
	SessionData *webauthn.SessionData
}

func initWebAuthn(rpDisplayName, rpID, rpOrigins string) error {
	webAuthnStateLock.Lock()
	defer webAuthnStateLock.Unlock()

	if webAuthnState != nil {
		return nil // Already initialized
	}

	config := &webauthn.Config{
		RPDisplayName: rpDisplayName,
		RPID:          rpID,
		RPOrigins:     []string{rpOrigins},
	}

	w, err := webauthn.New(config)
	if err != nil {
		return fmt.Errorf("failed to create WebAuthn: %w", err)
	}

	webAuthnState = &WebAuthnState{
		webauthn:     w,
		users:        make(map[string]*WebAuthnUser),
		nameToID:     make(map[string]string),
		credentials:  make(map[string][]webauthn.Credential),
		regSessions:  make(map[string]*RegistrationSession),
		authSessions: make(map[string]*AuthenticationSession),
	}

	return nil
}

func handleRequest(req *Request) *Response {
	resp := &Response{ID: req.ID}

	switch req.Method {
	case "webauthn.signup.begin":
		result, err := handleWebAuthnSignupBegin(req.Params)
		if err != nil {
			resp.Error = &ErrorObj{Code: -32603, Message: err.Error()}
		} else {
			resp.Result = result
		}

	case "webauthn.signup.finish":
		result, err := handleWebAuthnSignupFinish(req.Params)
		if err != nil {
			resp.Error = &ErrorObj{Code: -32603, Message: err.Error()}
		} else {
			resp.Result = result
		}

	case "webauthn.signin.begin":
		result, err := handleWebAuthnSigninBegin(req.Params)
		if err != nil {
			resp.Error = &ErrorObj{Code: -32603, Message: err.Error()}
		} else {
			resp.Result = result
		}

	case "webauthn.signin.finish":
		result, err := handleWebAuthnSigninFinish(req.Params)
		if err != nil {
			resp.Error = &ErrorObj{Code: -32603, Message: err.Error()}
		} else {
			resp.Result = result
		}

	case "webauthn.passkey.begin":
		result, err := handleWebAuthnPasskeyLoginBegin(req.Params)
		if err != nil {
			resp.Error = &ErrorObj{Code: -32603, Message: err.Error()}
		} else {
			resp.Result = result
		}

	case "webauthn.passkey.finish":
		result, err := handleWebAuthnPasskeyLoginFinish(req.Params)
		if err != nil {
			resp.Error = &ErrorObj{Code: -32603, Message: err.Error()}
		} else {
			resp.Result = result
		}

	case "webauthn.mfa.begin":
		result, err := handleWebAuthnMfaLoginBegin(req.Params)
		if err != nil {
			resp.Error = &ErrorObj{Code: -32603, Message: err.Error()}
		} else {
			resp.Result = result
		}

	case "webauthn.mfa.finish":
		result, err := handleWebAuthnMfaLoginFinish(req.Params)
		if err != nil {
			resp.Error = &ErrorObj{Code: -32603, Message: err.Error()}
		} else {
			resp.Result = result
		}

	case "webauthn.credentials.list":
		result, err := handleWebAuthnListCredentials(req.Params)
		if err != nil {
			resp.Error = &ErrorObj{Code: -32603, Message: err.Error()}
		} else {
			resp.Result = result
		}

	case "webauthn.credentials.delete":
		result, err := handleWebAuthnDeleteCredential(req.Params)
		if err != nil {
			resp.Error = &ErrorObj{Code: -32603, Message: err.Error()}
		} else {
			resp.Result = result
		}

	case "ed25519.generateKey":
		result, err := handleEd25519GenerateKey()
		if err != nil {
			resp.Error = &ErrorObj{Code: -32603, Message: err.Error()}
		} else {
			resp.Result = result
		}

	case "ed25519.sign":
		result, err := handleEd25519Sign(req.Params)
		if err != nil {
			resp.Error = &ErrorObj{Code: -32603, Message: err.Error()}
		} else {
			resp.Result = result
		}

	case "ed25519.verify":
		result, err := handleEd25519Verify(req.Params)
		if err != nil {
			resp.Error = &ErrorObj{Code: -32603, Message: err.Error()}
		} else {
			resp.Result = result
		}

	case "chacha20.encrypt":
		result, err := handleChaCha20Encrypt(req.Params)
		if err != nil {
			resp.Error = &ErrorObj{Code: -32603, Message: err.Error()}
		} else {
			resp.Result = result
		}

	case "chacha20.decrypt":
		result, err := handleChaCha20Decrypt(req.Params)
		if err != nil {
			resp.Error = &ErrorObj{Code: -32603, Message: err.Error()}
		} else {
			resp.Result = result
		}

	// System endpoints
	case "health":
		result, err := handleHealth()
		if err != nil {
			resp.Error = &ErrorObj{Code: -32603, Message: err.Error()}
		} else {
			resp.Result = result
		}

	case "metrics":
		result, err := handleMetrics()
		if err != nil {
			resp.Error = &ErrorObj{Code: -32603, Message: err.Error()}
		} else {
			resp.Result = result
		}

	case "version":
		result, err := handleVersion()
		if err != nil {
			resp.Error = &ErrorObj{Code: -32603, Message: err.Error()}
		} else {
			resp.Result = result
		}

	default:
		resp.Error = &ErrorObj{
			Code:    -32601,
			Message: fmt.Sprintf("Method not found: %s", req.Method),
		}
	}

	return resp
}

// Ed25519 handlers
func handleEd25519GenerateKey() (*Ed25519GenerateKeyResult, error) {
	pub, priv, err := ed25519.GenerateKey(rand.Reader)
	if err != nil {
		return nil, fmt.Errorf("failed to generate key: %w", err)
	}

	return &Ed25519GenerateKeyResult{
		PublicKey:  pub,
		PrivateKey: priv,
	}, nil
}

func handleEd25519Sign(params json.RawMessage) (*Ed25519SignResult, error) {
	var p Ed25519SignParams
	if err := json.Unmarshal(params, &p); err != nil {
		return nil, fmt.Errorf("invalid params: %w", err)
	}

	if len(p.PrivateKey) != ed25519.PrivateKeySize {
		return nil, fmt.Errorf("invalid private key size: expected %d, got %d", ed25519.PrivateKeySize, len(p.PrivateKey))
	}

	signature := ed25519.Sign(p.PrivateKey, p.Message)

	return &Ed25519SignResult{
		Signature: signature,
	}, nil
}

func handleEd25519Verify(params json.RawMessage) (*Ed25519VerifyResult, error) {
	var p Ed25519VerifyParams
	if err := json.Unmarshal(params, &p); err != nil {
		return nil, fmt.Errorf("invalid params: %w", err)
	}

	if len(p.PublicKey) != ed25519.PublicKeySize {
		return nil, fmt.Errorf("invalid public key size: expected %d, got %d", ed25519.PublicKeySize, len(p.PublicKey))
	}

	valid := ed25519.Verify(p.PublicKey, p.Message, p.Signature)

	return &Ed25519VerifyResult{
		Valid: valid,
	}, nil
}

// ChaCha20-Poly1305 handlers
func handleChaCha20Encrypt(params json.RawMessage) (*ChaCha20EncryptResult, error) {
	var p ChaCha20EncryptParams
	if err := json.Unmarshal(params, &p); err != nil {
		return nil, fmt.Errorf("invalid params: %w", err)
	}

	aead, err := chacha20poly1305.NewX(p.Key)
	if err != nil {
		return nil, fmt.Errorf("failed to create cipher: %w", err)
	}

	ciphertext := aead.Seal(nil, p.Nonce, p.Plaintext, p.AdditionalData)

	return &ChaCha20EncryptResult{
		Ciphertext: ciphertext,
	}, nil
}

func handleChaCha20Decrypt(params json.RawMessage) (*ChaCha20DecryptResult, error) {
	var p ChaCha20DecryptParams
	if err := json.Unmarshal(params, &p); err != nil {
		return nil, fmt.Errorf("invalid params: %w", err)
	}

	aead, err := chacha20poly1305.NewX(p.Key)
	if err != nil {
		return nil, fmt.Errorf("failed to create cipher: %w", err)
	}

	plaintext, err := aead.Open(nil, p.Nonce, p.Ciphertext, p.AdditionalData)
	if err != nil {
		return nil, fmt.Errorf("decryption failed: %w", err)
	}

	return &ChaCha20DecryptResult{
		Plaintext: plaintext,
	}, nil
}

// WebAuthn handlers
func handleWebAuthnSignupBegin(params json.RawMessage) (*WebAuthnSignupBeginResult, error) {
	var p WebAuthnSignupBeginParams
	if err := json.Unmarshal(params, &p); err != nil {
		return nil, fmt.Errorf("invalid params: %w", err)
	}

	// Initialize WebAuthn if not already done
	if err := initWebAuthn(p.RPDisplayName, p.RPID, p.RPOrigins); err != nil {
		return nil, fmt.Errorf("failed to initialize WebAuthn: %w", err)
	}

	webAuthnState.mu.Lock()
	defer webAuthnState.mu.Unlock()

	// Get or create user ID
	userID, exists := webAuthnState.nameToID[p.Username]
	if !exists {
		userID = uuid.New().String()
		webAuthnState.nameToID[p.Username] = userID
	}

	user := &WebAuthnUser{
		id:          []byte(userID),
		name:        p.Username,
		displayName: p.DisplayName,
	}

	// Prepare registration options based on scenario
	var opts []webauthn.RegistrationOption

	// Get existing credentials for exclusion
	existingCreds := webAuthnState.credentials[userID]
	if len(existingCreds) > 0 {
		credDescriptors := make([]protocol.CredentialDescriptor, len(existingCreds))
		for i, cred := range existingCreds {
			credDescriptors[i] = protocol.CredentialDescriptor{
				Type:         protocol.PublicKeyCredentialType,
				CredentialID: cred.ID,
			}
		}
		opts = append(opts, webauthn.WithExclusions(credDescriptors))
	}

	// Configure based on scenario
	switch p.Scenario {
	case "usernameless":
		opts = append(opts,
			webauthn.WithResidentKeyRequirement(protocol.ResidentKeyRequirementRequired),
			webauthn.WithAuthenticatorSelection(protocol.AuthenticatorSelection{
				RequireResidentKey: protocol.ResidentKeyRequired(),
				UserVerification:   protocol.VerificationRequired,
			}),
		)
	case "passwordless":
		opts = append(opts,
			webauthn.WithResidentKeyRequirement(protocol.ResidentKeyRequirementPreferred),
			webauthn.WithAuthenticatorSelection(protocol.AuthenticatorSelection{
				UserVerification: protocol.VerificationRequired,
			}),
		)
	case "mfa":
		opts = append(opts,
			webauthn.WithAuthenticatorSelection(protocol.AuthenticatorSelection{
				UserVerification: protocol.VerificationDiscouraged,
			}),
		)
	default:
		opts = append(opts,
			webauthn.WithResidentKeyRequirement(protocol.ResidentKeyRequirementPreferred),
		)
	}

	// Begin registration
	creation, sessionData, err := webAuthnState.webauthn.BeginMediatedRegistration(user, protocol.MediationDefault, opts...)
	if err != nil {
		return nil, fmt.Errorf("failed to begin registration: %w", err)
	}

	// Store session
	sessionID := uuid.New().String()
	webAuthnState.regSessions[sessionID] = &RegistrationSession{
		UserID:      userID,
		Username:    p.Username,
		SessionData: sessionData,
	}
	webAuthnState.users[userID] = user

	// Serialize challenge
	challengeJSON, err := json.Marshal(creation)
	if err != nil {
		return nil, fmt.Errorf("failed to marshal challenge: %w", err)
	}

	return &WebAuthnSignupBeginResult{
		SessionID:     sessionID,
		ChallengeJSON: string(challengeJSON),
	}, nil
}

func handleWebAuthnSignupFinish(params json.RawMessage) (*WebAuthnSignupFinishResult, error) {
	var p WebAuthnSignupFinishParams
	if err := json.Unmarshal(params, &p); err != nil {
		return nil, fmt.Errorf("invalid params: %w", err)
	}

	webAuthnState.mu.Lock()
	defer webAuthnState.mu.Unlock()

	// Get session
	session, exists := webAuthnState.regSessions[p.SessionID]
	if !exists {
		return nil, fmt.Errorf("session not found")
	}
	defer delete(webAuthnState.regSessions, p.SessionID)

	// Get user
	user, exists := webAuthnState.users[session.UserID]
	if !exists {
		return nil, fmt.Errorf("user not found")
	}

	// Parse credential
	var ccr protocol.CredentialCreationResponse
	if err := json.Unmarshal([]byte(p.CredentialJSON), &ccr); err != nil {
		return nil, fmt.Errorf("failed to parse credential: %w", err)
	}

	parsedResponse, err := ccr.Parse()
	if err != nil {
		return nil, fmt.Errorf("failed to parse credential response: %w", err)
	}

	// Finish registration
	credential, err := webAuthnState.webauthn.CreateCredential(user, *session.SessionData, parsedResponse)
	if err != nil {
		return nil, fmt.Errorf("failed to create credential: %w", err)
	}

	// Store credential
	webAuthnState.credentials[session.UserID] = append(webAuthnState.credentials[session.UserID], *credential)

	return &WebAuthnSignupFinishResult{
		UserID:       session.UserID,
		CredentialID: string(credential.ID),
	}, nil
}

func handleWebAuthnSigninBegin(params json.RawMessage) (*WebAuthnSigninBeginResult, error) {
	var p WebAuthnSigninBeginParams
	if err := json.Unmarshal(params, &p); err != nil {
		return nil, fmt.Errorf("invalid params: %w", err)
	}

	if webAuthnState == nil {
		return nil, fmt.Errorf("WebAuthn not initialized")
	}

	webAuthnState.mu.RLock()
	defer webAuthnState.mu.RUnlock()

	// Get user
	userID, exists := webAuthnState.nameToID[p.Username]
	if !exists {
		return nil, fmt.Errorf("user not found")
	}

	user, exists := webAuthnState.users[userID]
	if !exists {
		return nil, fmt.Errorf("user not found")
	}

	// Get credentials
	credentials := webAuthnState.credentials[userID]
	if len(credentials) == 0 {
		return nil, fmt.Errorf("no credentials found for user")
	}

	// Create a user with credentials for authentication
	userWithCreds := &struct {
		*WebAuthnUser
		creds []webauthn.Credential
	}{
		WebAuthnUser: user,
		creds:        credentials,
	}

	// Override WebAuthnCredentials method
	credGetter := func() []webauthn.Credential {
		return userWithCreds.creds
	}
	_ = credGetter // Will use in assertion

	// Begin authentication
	var opts []webauthn.LoginOption
	assertion, sessionData, err := webAuthnState.webauthn.BeginMediatedLogin(user, protocol.MediationDefault, opts...)
	if err != nil {
		return nil, fmt.Errorf("failed to begin login: %w", err)
	}

	// Store session
	sessionID := uuid.New().String()
	webAuthnState.mu.RUnlock()
	webAuthnState.mu.Lock()
	webAuthnState.authSessions[sessionID] = &AuthenticationSession{
		UserID:      userID,
		SessionData: sessionData,
	}
	webAuthnState.mu.Unlock()
	webAuthnState.mu.RLock()

	// Serialize challenge
	challengeJSON, err := json.Marshal(assertion)
	if err != nil {
		return nil, fmt.Errorf("failed to marshal challenge: %w", err)
	}

	return &WebAuthnSigninBeginResult{
		SessionID:     sessionID,
		ChallengeJSON: string(challengeJSON),
	}, nil
}

func handleWebAuthnSigninFinish(params json.RawMessage) (*WebAuthnSigninFinishResult, error) {
	var p WebAuthnSigninFinishParams
	if err := json.Unmarshal(params, &p); err != nil {
		return nil, fmt.Errorf("invalid params: %w", err)
	}

	webAuthnState.mu.Lock()
	defer webAuthnState.mu.Unlock()

	// Get session
	session, exists := webAuthnState.authSessions[p.SessionID]
	if !exists {
		return nil, fmt.Errorf("session not found")
	}
	defer delete(webAuthnState.authSessions, p.SessionID)

	// Get user
	user, exists := webAuthnState.users[session.UserID]
	if !exists {
		return nil, fmt.Errorf("user not found")
	}

	// Get credentials
	credentials := webAuthnState.credentials[session.UserID]

	// Parse credential assertion
	var car protocol.CredentialAssertionResponse
	if err := json.Unmarshal([]byte(p.CredentialJSON), &car); err != nil {
		return nil, fmt.Errorf("failed to parse credential: %w", err)
	}

	parsedResponse, err := car.Parse()
	if err != nil {
		return nil, fmt.Errorf("failed to parse credential response: %w", err)
	}

	// Verify assertion
	var foundCred *webauthn.Credential
	for i := range credentials {
		if string(credentials[i].ID) == string(parsedResponse.RawID) {
			foundCred = &credentials[i]
			break
		}
	}

	if foundCred == nil {
		return nil, fmt.Errorf("credential not found")
	}

	// Validate the credential
	_, err = webAuthnState.webauthn.ValidateLogin(user, *session.SessionData, parsedResponse)
	if err != nil {
		return nil, fmt.Errorf("failed to validate login: %w", err)
	}

	// Update credential (signature counter, etc.)
	for i := range credentials {
		if string(credentials[i].ID) == string(parsedResponse.RawID) {
			webAuthnState.credentials[session.UserID][i] = *foundCred
			break
		}
	}

	return &WebAuthnSigninFinishResult{
		UserID:   session.UserID,
		Username: user.name,
	}, nil
}

// Passkey Login handlers (Discoverable Credentials)
func handleWebAuthnPasskeyLoginBegin(params json.RawMessage) (*WebAuthnPasskeyLoginBeginResult, error) {
	var p WebAuthnPasskeyLoginBeginParams
	if err := json.Unmarshal(params, &p); err != nil {
		return nil, fmt.Errorf("invalid params: %w", err)
	}

	// Initialize WebAuthn if not already done
	if err := initWebAuthn(p.RPDisplayName, p.RPID, p.RPOrigins); err != nil {
		return nil, fmt.Errorf("failed to initialize WebAuthn: %w", err)
	}

	// For discoverable credentials (passkey login), we create a challenge
	// without specifying allowed credentials - the browser will show all
	// available passkeys and the user info comes from the credential

	// Generate challenge
	challenge, err := protocol.CreateChallenge()
	if err != nil {
		return nil, fmt.Errorf("failed to create challenge: %w", err)
	}

	// Create assertion options for discoverable credentials
	assertion := &protocol.CredentialAssertion{
		Response: protocol.PublicKeyCredentialRequestOptions{
			Challenge:        challenge,
			RelyingPartyID:   p.RPID,
			UserVerification: protocol.VerificationRequired,
			// Empty AllowedCredentials means discoverable (any credential works)
			AllowedCredentials: []protocol.CredentialDescriptor{},
		},
	}

	// Create session data
	sessionData := &webauthn.SessionData{
		Challenge:        challenge.String(),
		UserID:           []byte{}, // Will be filled from credential
		UserVerification: protocol.VerificationRequired,
	}

	// Store session
	sessionID := uuid.New().String()
	webAuthnState.mu.Lock()
	webAuthnState.authSessions[sessionID] = &AuthenticationSession{
		UserID:      "", // Will be determined from credential
		SessionData: sessionData,
	}
	webAuthnState.mu.Unlock()

	// Serialize challenge
	challengeJSON, err := json.Marshal(assertion)
	if err != nil {
		return nil, fmt.Errorf("failed to marshal challenge: %w", err)
	}

	return &WebAuthnPasskeyLoginBeginResult{
		SessionID:     sessionID,
		ChallengeJSON: string(challengeJSON),
	}, nil
}

func handleWebAuthnPasskeyLoginFinish(params json.RawMessage) (*WebAuthnPasskeyLoginFinishResult, error) {
	var p WebAuthnPasskeyLoginFinishParams
	if err := json.Unmarshal(params, &p); err != nil {
		return nil, fmt.Errorf("invalid params: %w", err)
	}

	webAuthnState.mu.Lock()
	defer webAuthnState.mu.Unlock()

	// Get session
	session, exists := webAuthnState.authSessions[p.SessionID]
	if !exists {
		return nil, fmt.Errorf("session not found")
	}
	defer delete(webAuthnState.authSessions, p.SessionID)

	// Parse credential assertion
	var car protocol.CredentialAssertionResponse
	if err := json.Unmarshal([]byte(p.CredentialJSON), &car); err != nil {
		return nil, fmt.Errorf("failed to parse credential: %w", err)
	}

	parsedResponse, err := car.Parse()
	if err != nil {
		return nil, fmt.Errorf("failed to parse credential response: %w", err)
	}

	// Find the credential across all users
	var foundUser *WebAuthnUser
	var foundCred *webauthn.Credential
	for userID, credentials := range webAuthnState.credentials {
		for i := range credentials {
			if string(credentials[i].ID) == string(parsedResponse.RawID) {
				foundCred = &credentials[i]
				foundUser = webAuthnState.users[userID]
				session.UserID = userID
				break
			}
		}
		if foundCred != nil {
			break
		}
	}

	if foundCred == nil || foundUser == nil {
		return nil, fmt.Errorf("credential not found")
	}

	// Validate the credential
	_, err = webAuthnState.webauthn.ValidateLogin(foundUser, *session.SessionData, parsedResponse)
	if err != nil {
		return nil, fmt.Errorf("failed to validate passkey login: %w", err)
	}

	// Update credential counter
	for i := range webAuthnState.credentials[session.UserID] {
		if string(webAuthnState.credentials[session.UserID][i].ID) == string(parsedResponse.RawID) {
			webAuthnState.credentials[session.UserID][i] = *foundCred
			break
		}
	}

	return &WebAuthnPasskeyLoginFinishResult{
		UserID:   session.UserID,
		Username: foundUser.name,
	}, nil
}

// MFA Login handlers
func handleWebAuthnMfaLoginBegin(params json.RawMessage) (*WebAuthnMfaLoginBeginResult, error) {
	var p WebAuthnMfaLoginBeginParams
	if err := json.Unmarshal(params, &p); err != nil {
		return nil, fmt.Errorf("invalid params: %w", err)
	}

	if webAuthnState == nil {
		return nil, fmt.Errorf("WebAuthn not initialized")
	}

	webAuthnState.mu.RLock()
	defer webAuthnState.mu.RUnlock()

	// Get user
	userID, exists := webAuthnState.nameToID[p.Username]
	if !exists {
		return nil, fmt.Errorf("user not found")
	}

	user, exists := webAuthnState.users[userID]
	if !exists {
		return nil, fmt.Errorf("user not found")
	}

	// Begin MFA login
	var opts []webauthn.LoginOption
	assertion, sessionData, err := webAuthnState.webauthn.BeginLogin(user, opts...)
	if err != nil {
		return nil, fmt.Errorf("failed to begin MFA login: %w", err)
	}

	// Store session
	sessionID := uuid.New().String()
	webAuthnState.mu.RUnlock()
	webAuthnState.mu.Lock()
	webAuthnState.authSessions[sessionID] = &AuthenticationSession{
		UserID:      userID,
		SessionData: sessionData,
	}
	webAuthnState.mu.Unlock()
	webAuthnState.mu.RLock()

	// Serialize challenge
	challengeJSON, err := json.Marshal(assertion)
	if err != nil {
		return nil, fmt.Errorf("failed to marshal challenge: %w", err)
	}

	return &WebAuthnMfaLoginBeginResult{
		SessionID:     sessionID,
		ChallengeJSON: string(challengeJSON),
	}, nil
}

func handleWebAuthnMfaLoginFinish(params json.RawMessage) (*WebAuthnMfaLoginFinishResult, error) {
	var p WebAuthnMfaLoginFinishParams
	if err := json.Unmarshal(params, &p); err != nil {
		return nil, fmt.Errorf("invalid params: %w", err)
	}

	webAuthnState.mu.Lock()
	defer webAuthnState.mu.Unlock()

	// Get session
	session, exists := webAuthnState.authSessions[p.SessionID]
	if !exists {
		return nil, fmt.Errorf("session not found")
	}
	defer delete(webAuthnState.authSessions, p.SessionID)

	// Get user
	user, exists := webAuthnState.users[session.UserID]
	if !exists {
		return nil, fmt.Errorf("user not found")
	}

	// Parse credential assertion
	var car protocol.CredentialAssertionResponse
	if err := json.Unmarshal([]byte(p.CredentialJSON), &car); err != nil {
		return nil, fmt.Errorf("failed to parse credential: %w", err)
	}

	parsedResponse, err := car.Parse()
	if err != nil {
		return nil, fmt.Errorf("failed to parse credential response: %w", err)
	}

	// Find credential
	credentials := webAuthnState.credentials[session.UserID]
	var foundCred *webauthn.Credential
	for i := range credentials {
		if string(credentials[i].ID) == string(parsedResponse.RawID) {
			foundCred = &credentials[i]
			break
		}
	}

	if foundCred == nil {
		return nil, fmt.Errorf("credential not found")
	}

	// Validate MFA
	_, err = webAuthnState.webauthn.ValidateLogin(user, *session.SessionData, parsedResponse)
	if err != nil {
		return nil, fmt.Errorf("failed to validate MFA: %w", err)
	}

	// Update credential counter
	for i := range webAuthnState.credentials[session.UserID] {
		if string(webAuthnState.credentials[session.UserID][i].ID) == string(parsedResponse.RawID) {
			webAuthnState.credentials[session.UserID][i] = *foundCred
			break
		}
	}

	return &WebAuthnMfaLoginFinishResult{
		UserID:   session.UserID,
		Username: user.name,
		Verified: true,
	}, nil
}

// Credential Management handlers
func handleWebAuthnListCredentials(params json.RawMessage) (*WebAuthnListCredentialsResult, error) {
	var p WebAuthnListCredentialsParams
	if err := json.Unmarshal(params, &p); err != nil {
		return nil, fmt.Errorf("invalid params: %w", err)
	}

	if webAuthnState == nil {
		return nil, fmt.Errorf("WebAuthn not initialized")
	}

	webAuthnState.mu.RLock()
	defer webAuthnState.mu.RUnlock()

	// Get user
	userID, exists := webAuthnState.nameToID[p.Username]
	if !exists {
		return nil, fmt.Errorf("user not found")
	}

	// Get credentials
	credentials := webAuthnState.credentials[userID]
	result := &WebAuthnListCredentialsResult{
		Credentials: make([]WebAuthnCredentialInfo, 0, len(credentials)),
	}

	for _, cred := range credentials {
		info := WebAuthnCredentialInfo{
			ID:              base64.StdEncoding.EncodeToString(cred.ID),
			PublicKey:       base64.StdEncoding.EncodeToString(cred.PublicKey),
			SignCount:       cred.Authenticator.SignCount,
			AAGUID:          base64.StdEncoding.EncodeToString(cred.Authenticator.AAGUID),
			AttestationType: cred.AttestationType,
		}
		result.Credentials = append(result.Credentials, info)
	}

	return result, nil
}

func handleWebAuthnDeleteCredential(params json.RawMessage) (*WebAuthnDeleteCredentialResult, error) {
	var p WebAuthnDeleteCredentialParams
	if err := json.Unmarshal(params, &p); err != nil {
		return nil, fmt.Errorf("invalid params: %w", err)
	}

	if webAuthnState == nil {
		return nil, fmt.Errorf("WebAuthn not initialized")
	}

	webAuthnState.mu.Lock()
	defer webAuthnState.mu.Unlock()

	// Get user
	userID, exists := webAuthnState.nameToID[p.Username]
	if !exists {
		return nil, fmt.Errorf("user not found")
	}

	// Decode credential ID
	credID, err := base64.StdEncoding.DecodeString(p.CredentialID)
	if err != nil {
		return nil, fmt.Errorf("invalid credential ID: %w", err)
	}

	// Find and remove credential
	credentials := webAuthnState.credentials[userID]
	newCreds := make([]webauthn.Credential, 0, len(credentials))
	found := false

	for _, cred := range credentials {
		if string(cred.ID) != string(credID) {
			newCreds = append(newCreds, cred)
		} else {
			found = true
		}
	}

	if !found {
		return nil, fmt.Errorf("credential not found")
	}

	webAuthnState.credentials[userID] = newCreds

	return &WebAuthnDeleteCredentialResult{
		Success: true,
	}, nil
}

// System endpoint handlers

// Version information
const (
	Version   = "2.0.0"
	BuildDate = "2026-06-26"
)

type HealthResult struct {
	Status  string  `json:"status"`
	Uptime  float64 `json:"uptime_seconds"`
	Version string  `json:"version"`
}

type VersionResult struct {
	Version   string `json:"version"`
	GoVersion string `json:"go_version"`
	BuildDate string `json:"build_date"`
}

func handleHealth() (*HealthResult, error) {
	metrics := GetMetrics()
	return &HealthResult{
		Status:  "ok",
		Uptime:  metrics.UptimeSeconds,
		Version: Version,
	}, nil
}

func handleMetrics() (*MetricsSnapshot, error) {
	metrics := GetMetrics()
	return &metrics, nil
}

func handleVersion() (*VersionResult, error) {
	return &VersionResult{
		Version:   Version,
		GoVersion: runtime.Version(),
		BuildDate: BuildDate,
	}, nil
}
