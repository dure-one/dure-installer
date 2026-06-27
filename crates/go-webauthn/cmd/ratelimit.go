package main

import (
	"sync"
	"time"
)

// RateLimiter implements token bucket rate limiting
type RateLimiter struct {
	buckets map[string]*TokenBucket
	mu      sync.RWMutex

	// Configuration
	perUserLimit   int           // Requests per window
	perMethodLimit int           // Requests per method per window
	window         time.Duration // Time window
}

// TokenBucket represents a token bucket for rate limiting
type TokenBucket struct {
	tokens    int
	capacity  int
	lastReset time.Time
	window    time.Duration
}

var globalRateLimiter *RateLimiter

func init() {
	// Default: 100 requests per minute per user, 50 per method
	globalRateLimiter = NewRateLimiter(100, 50, time.Minute)
}

// NewRateLimiter creates a new rate limiter
func NewRateLimiter(perUserLimit, perMethodLimit int, window time.Duration) *RateLimiter {
	return &RateLimiter{
		buckets:        make(map[string]*TokenBucket),
		perUserLimit:   perUserLimit,
		perMethodLimit: perMethodLimit,
		window:         window,
	}
}

// getBucket gets or creates a token bucket
func (rl *RateLimiter) getBucket(key string, capacity int) *TokenBucket {
	rl.mu.Lock()
	defer rl.mu.Unlock()

	bucket, exists := rl.buckets[key]
	if !exists {
		bucket = &TokenBucket{
			tokens:    capacity,
			capacity:  capacity,
			lastReset: time.Now(),
			window:    rl.window,
		}
		rl.buckets[key] = bucket
	}

	return bucket
}

// Allow checks if a request is allowed
func (rl *RateLimiter) Allow(username string, method string) bool {
	now := time.Now()

	// Check user-level rate limit
	userKey := "user:" + username
	userBucket := rl.getBucket(userKey, rl.perUserLimit)
	if !userBucket.consume(now) {
		Debug("Rate limit exceeded for user", map[string]interface{}{
			"username": username,
			"limit":    rl.perUserLimit,
		})
		return false
	}

	// Check method-level rate limit
	methodKey := "method:" + username + ":" + method
	methodBucket := rl.getBucket(methodKey, rl.perMethodLimit)
	if !methodBucket.consume(now) {
		Debug("Rate limit exceeded for method", map[string]interface{}{
			"username": username,
			"method":   method,
			"limit":    rl.perMethodLimit,
		})
		// Refund the user-level token since method limit blocked it
		userBucket.refund()
		return false
	}

	return true
}

// consume attempts to consume a token
func (tb *TokenBucket) consume(now time.Time) bool {
	// Reset if window has passed
	if now.Sub(tb.lastReset) > tb.window {
		tb.tokens = tb.capacity
		tb.lastReset = now
	}

	if tb.tokens > 0 {
		tb.tokens--
		return true
	}

	return false
}

// refund refunds a token (called when a request is rejected for other reasons)
func (tb *TokenBucket) refund() {
	if tb.tokens < tb.capacity {
		tb.tokens++
	}
}

// CheckRateLimit checks if a request is within rate limits
func CheckRateLimit(username string, method string) bool {
	if username == "" {
		username = "anonymous"
	}
	return globalRateLimiter.Allow(username, method)
}

// SetRateLimits configures global rate limits
func SetRateLimits(perUserLimit, perMethodLimit int, window time.Duration) {
	globalRateLimiter = NewRateLimiter(perUserLimit, perMethodLimit, window)
}

// CleanupRateLimiters removes old buckets (should be called periodically)
func CleanupRateLimiters() {
	globalRateLimiter.mu.Lock()
	defer globalRateLimiter.mu.Unlock()

	now := time.Now()
	for key, bucket := range globalRateLimiter.buckets {
		// Remove buckets that haven't been used in 2x the window
		if now.Sub(bucket.lastReset) > globalRateLimiter.window*2 {
			delete(globalRateLimiter.buckets, key)
		}
	}

	Debug("Rate limiter cleanup", map[string]interface{}{
		"remaining_buckets": len(globalRateLimiter.buckets),
	})
}
