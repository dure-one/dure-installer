package main

import (
	"testing"
	"time"
)

func TestMetricsRecordRequest(t *testing.T) {
	m := NewMetrics()

	// Record a successful request
	m.RecordRequest("test.method", 100*time.Millisecond, true)

	snapshot := m.GetMetrics()

	if snapshot.RequestsTotal != 1 {
		t.Errorf("Expected RequestsTotal=1, got %d", snapshot.RequestsTotal)
	}
	if snapshot.RequestsSuccess != 1 {
		t.Errorf("Expected RequestsSuccess=1, got %d", snapshot.RequestsSuccess)
	}
	if snapshot.RequestsError != 0 {
		t.Errorf("Expected RequestsError=0, got %d", snapshot.RequestsError)
	}
}

func TestMetricsRecordError(t *testing.T) {
	m := NewMetrics()

	// Record a failed request
	m.RecordRequest("test.method", 50*time.Millisecond, false)

	snapshot := m.GetMetrics()

	if snapshot.RequestsError != 1 {
		t.Errorf("Expected RequestsError=1, got %d", snapshot.RequestsError)
	}
	if snapshot.ErrorRate != 1.0 {
		t.Errorf("Expected ErrorRate=1.0, got %f", snapshot.ErrorRate)
	}
}

func TestMetricsPerMethod(t *testing.T) {
	m := NewMetrics()

	// Record requests for different methods
	m.RecordRequest("method1", 10*time.Millisecond, true)
	m.RecordRequest("method2", 20*time.Millisecond, true)
	m.RecordRequest("method1", 30*time.Millisecond, false)

	snapshot := m.GetMetrics()

	if len(snapshot.MethodStats) != 2 {
		t.Errorf("Expected 2 methods, got %d", len(snapshot.MethodStats))
	}

	method1Stats := snapshot.MethodStats["method1"]
	if method1Stats.Total != 2 {
		t.Errorf("Expected method1 Total=2, got %d", method1Stats.Total)
	}
	if method1Stats.Success != 1 {
		t.Errorf("Expected method1 Success=1, got %d", method1Stats.Success)
	}
	if method1Stats.Error != 1 {
		t.Errorf("Expected method1 Error=1, got %d", method1Stats.Error)
	}
}

func TestMetricsNoNaN(t *testing.T) {
	m := NewMetrics()

	// Get metrics with no requests (should not produce NaN)
	snapshot := m.GetMetrics()

	if snapshot.AvgLatencyMs != 0.0 {
		t.Errorf("Expected AvgLatencyMs=0.0 for empty metrics, got %f", snapshot.AvgLatencyMs)
	}
	if snapshot.ErrorRate != 0.0 {
		t.Errorf("Expected ErrorRate=0.0 for empty metrics, got %f", snapshot.ErrorRate)
	}
}

func TestMetricsLatencyCalculation(t *testing.T) {
	m := NewMetrics()

	// Record requests with known latencies
	m.RecordRequest("test", 100*time.Millisecond, true) // 100ms
	m.RecordRequest("test", 200*time.Millisecond, true) // 200ms

	snapshot := m.GetMetrics()

	// Average should be 150ms
	expected := 150.0
	tolerance := 1.0 // 1ms tolerance for floating point
	if snapshot.AvgLatencyMs < expected-tolerance || snapshot.AvgLatencyMs > expected+tolerance {
		t.Errorf("Expected AvgLatencyMs≈%.1f, got %.1f", expected, snapshot.AvgLatencyMs)
	}
}
