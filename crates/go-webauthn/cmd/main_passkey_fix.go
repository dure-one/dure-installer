// Passkey Login handlers (Discoverable Credentials) - SIMPLIFIED
func handleWebAuthnPasskeyLoginBegin(params json.RawMessage) (*WebAuthnPasskeyLoginBeginResult, error) {
	var p WebAuthnPasskeyLoginBeginParams
	if err := json.Unmarshal(params, &p); err != nil {
		return nil, fmt.Errorf("invalid params: %w", err)
	}

	// Initialize WebAuthn if not already done
	if err := initWebAuthn(p.RPDisplayName, p.RPID, p.RPOrigins); err != nil {
		return nil, fmt.Errorf("failed to initialize WebAuthn: %w", err)
	}

	webAuthnState.mu.RLock()
	defer webAuthnState.mu.RUnlock()

	// For discoverable credentials, create assertion without specific user
	var opts []webauthn.LoginOption

	// Begin login (will work with any credential for this RP)
	assertion, sessionData, err := webAuthnState.webauthn.BeginLogin(nil, opts...)
	if err != nil {
		return nil, fmt.Errorf("failed to begin passkey login: %w", err)
	}

	// Store session
	sessionID := uuid.New().String()
	webAuthnState.mu.RUnlock()
	webAuthnState.mu.Lock()
	webAuthnState.authSessions[sessionID] = &AuthenticationSession{
		UserID:      "", // Will be determined from credential
		SessionData: sessionData,
	}
	webAuthnState.mu.Unlock()
	webAuthnState.mu.RLock()

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

	// Begin MFA login (same as regular login)
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
