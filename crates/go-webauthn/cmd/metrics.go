package main

import (
	"sync"
	"sync/atomic"
	"time"
)

// Metrics tracks operational metrics
type Metrics struct {
	StartTime time.Time

	// Request counters
	RequestsTotal   uint64
	RequestsSuccess uint64
	RequestsError   uint64

	// Per-method counters
	methodCounts map[string]*MethodMetrics
	mu           sync.RWMutex

	// Latency tracking (nanoseconds)
	totalLatency uint64
}

// MethodMetrics tracks metrics per method
type MethodMetrics struct {
	Total   uint64
	Success uint64
	Error   uint64
	Latency uint64 // Total latency in nanoseconds
}

var globalMetrics *Metrics

func init() {
	globalMetrics = NewMetrics()
}

// NewMetrics creates a new metrics tracker
func NewMetrics() *Metrics {
	return &Metrics{
		StartTime:    time.Now(),
		methodCounts: make(map[string]*MethodMetrics),
	}
}

// RecordRequest records a request
func (m *Metrics) RecordRequest(method string, duration time.Duration, success bool) {
	// Update global counters
	atomic.AddUint64(&m.RequestsTotal, 1)
	if success {
		atomic.AddUint64(&m.RequestsSuccess, 1)
	} else {
		atomic.AddUint64(&m.RequestsError, 1)
	}
	atomic.AddUint64(&m.totalLatency, uint64(duration.Nanoseconds()))

	// Update per-method counters
	m.mu.Lock()
	defer m.mu.Unlock()

	methodMetrics, exists := m.methodCounts[method]
	if !exists {
		methodMetrics = &MethodMetrics{}
		m.methodCounts[method] = methodMetrics
	}

	atomic.AddUint64(&methodMetrics.Total, 1)
	if success {
		atomic.AddUint64(&methodMetrics.Success, 1)
	} else {
		atomic.AddUint64(&methodMetrics.Error, 1)
	}
	atomic.AddUint64(&methodMetrics.Latency, uint64(duration.Nanoseconds()))
}

// GetMetrics returns current metrics snapshot
func (m *Metrics) GetMetrics() MetricsSnapshot {
	m.mu.RLock()
	defer m.mu.RUnlock()

	uptime := time.Since(m.StartTime)
	total := atomic.LoadUint64(&m.RequestsTotal)
	success := atomic.LoadUint64(&m.RequestsSuccess)
	errors := atomic.LoadUint64(&m.RequestsError)
	totalLatency := atomic.LoadUint64(&m.totalLatency)

	avgLatencyMs := 0.0
	if total > 0 {
		avgLatencyMs = float64(totalLatency) / float64(total) / 1_000_000
	}

	methodStats := make(map[string]MethodStats)
	for method, metrics := range m.methodCounts {
		methodTotal := atomic.LoadUint64(&metrics.Total)
		methodSuccess := atomic.LoadUint64(&metrics.Success)
		methodError := atomic.LoadUint64(&metrics.Error)
		methodLatency := atomic.LoadUint64(&metrics.Latency)

		methodAvgLatencyMs := 0.0
		if methodTotal > 0 {
			methodAvgLatencyMs = float64(methodLatency) / float64(methodTotal) / 1_000_000
		}

		methodErrorRate := 0.0
		if methodTotal > 0 {
			methodErrorRate = float64(methodError) / float64(methodTotal)
		}

		methodStats[method] = MethodStats{
			Total:        methodTotal,
			Success:      methodSuccess,
			Error:        methodError,
			AvgLatencyMs: methodAvgLatencyMs,
			ErrorRate:    methodErrorRate,
		}
	}

	errorRate := 0.0
	if total > 0 {
		errorRate = float64(errors) / float64(total)
	}

	return MetricsSnapshot{
		UptimeSeconds:   uptime.Seconds(),
		RequestsTotal:   total,
		RequestsSuccess: success,
		RequestsError:   errors,
		ErrorRate:       errorRate,
		AvgLatencyMs:    avgLatencyMs,
		MethodStats:     methodStats,
	}
}

// MetricsSnapshot is a point-in-time metrics snapshot
type MetricsSnapshot struct {
	UptimeSeconds   float64                `json:"uptime_seconds"`
	RequestsTotal   uint64                 `json:"requests_total"`
	RequestsSuccess uint64                 `json:"requests_success"`
	RequestsError   uint64                 `json:"requests_error"`
	ErrorRate       float64                `json:"error_rate"`
	AvgLatencyMs    float64                `json:"avg_latency_ms"`
	MethodStats     map[string]MethodStats `json:"method_stats"`
}

// MethodStats contains per-method statistics
type MethodStats struct {
	Total        uint64  `json:"total"`
	Success      uint64  `json:"success"`
	Error        uint64  `json:"error"`
	AvgLatencyMs float64 `json:"avg_latency_ms"`
	ErrorRate    float64 `json:"error_rate"`
}

// Global functions for convenience
func RecordRequest(method string, duration time.Duration, success bool) {
	globalMetrics.RecordRequest(method, duration, success)
}

func GetMetrics() MetricsSnapshot {
	return globalMetrics.GetMetrics()
}
