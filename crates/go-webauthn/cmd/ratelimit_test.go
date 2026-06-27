package main

import (
	"testing"
	"time"
)

func TestRateLimiterAllow(t *testing.T) {
	rl := NewRateLimiter(5, 5, time.Minute) // Set method limit same as user limit

	// First 5 requests should be allowed (user has quota of 5, method has quota of 5)
	for i := 0; i < 5; i++ {
		if !rl.Allow("testuser-allow", "test.method") {
			t.Errorf("Request %d should be allowed", i+1)
		}
	}

	// 6th request should be blocked
	if rl.Allow("testuser-allow", "test.method") {
		t.Error("6th request should be blocked by user limit")
	}
}

func TestRateLimiterPerMethod(t *testing.T) {
	rl := NewRateLimiter(10, 2, time.Minute)

	// First 2 requests to method1 should be allowed
	if !rl.Allow("testuser", "method1") {
		t.Error("Request 1 to method1 should be allowed")
	}
	if !rl.Allow("testuser", "method1") {
		t.Error("Request 2 to method1 should be allowed")
	}

	// 3rd request to method1 should be blocked by method limit
	if rl.Allow("testuser", "method1") {
		t.Error("Request 3 to method1 should be blocked by method limit")
	}

	// But requests to method2 should still be allowed (different method)
	if !rl.Allow("testuser", "method2") {
		t.Error("Request 1 to method2 should be allowed")
	}
}

func TestRateLimiterReset(t *testing.T) {
	rl := NewRateLimiter(2, 2, 100*time.Millisecond)

	// Use up the quota
	rl.Allow("testuser", "test.method")
	rl.Allow("testuser", "test.method")

	// Should be blocked
	if rl.Allow("testuser", "test.method") {
		t.Error("Request should be blocked")
	}

	// Wait for window to reset
	time.Sleep(150 * time.Millisecond)

	// Should be allowed again after reset
	if !rl.Allow("testuser", "test.method") {
		t.Error("Request should be allowed after window reset")
	}
}

func TestRateLimiterDifferentUsers(t *testing.T) {
	rl := NewRateLimiter(2, 2, time.Minute)

	// User1 uses quota
	rl.Allow("user1", "test.method")
	rl.Allow("user1", "test.method")

	// User1 should be blocked
	if rl.Allow("user1", "test.method") {
		t.Error("user1 should be blocked")
	}

	// User2 should still be allowed (separate quota)
	if !rl.Allow("user2", "test.method") {
		t.Error("user2 should be allowed (different user)")
	}
}

func TestCheckRateLimit(t *testing.T) {
	// Reset global rate limiter with small limits for testing
	SetRateLimits(3, 2, time.Minute)

	// Test the global convenience function
	if !CheckRateLimit("testuser", "test.method") {
		t.Error("First request should be allowed")
	}
	if !CheckRateLimit("testuser", "test.method") {
		t.Error("Second request should be allowed")
	}

	// Third request hits method limit (2 per method)
	if CheckRateLimit("testuser", "test.method") {
		t.Error("Third request should be blocked by method limit")
	}

	// Reset for next test
	SetRateLimits(100, 50, time.Minute)
}

func TestRateLimiterAnonymous(t *testing.T) {
	// Reset global rate limiter with small limits for testing
	SetRateLimits(2, 2, time.Minute)

	// Empty username should be treated as "anonymous"
	if !CheckRateLimit("", "test.method") {
		t.Error("Anonymous request should be allowed")
	}
	if !CheckRateLimit("", "test.method") {
		t.Error("Anonymous request 2 should be allowed")
	}

	// Reset for other tests
	SetRateLimits(100, 50, time.Minute)
}
