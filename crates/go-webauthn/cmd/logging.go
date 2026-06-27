package main

import (
	"fmt"
	"log"
	"os"
	"time"
)

// LogLevel represents logging level
type LogLevel int

const (
	LogLevelDebug LogLevel = iota
	LogLevelInfo
	LogLevelWarn
	LogLevelError
)

// Logger provides structured logging
type Logger struct {
	level  LogLevel
	logger *log.Logger
}

var globalLogger *Logger

func init() {
	globalLogger = NewLogger(LogLevelInfo)
}

// NewLogger creates a new logger
func NewLogger(level LogLevel) *Logger {
	return &Logger{
		level:  level,
		logger: log.New(os.Stderr, "", 0),
	}
}

// SetLogLevel sets the global log level
func SetLogLevel(level LogLevel) {
	globalLogger.level = level
}

// EnableDebugMode enables debug logging
func EnableDebugMode() {
	SetLogLevel(LogLevelDebug)
	globalLogger.Info("Debug mode enabled")
}

// IsDebugEnabled returns true if debug logging is enabled
func IsDebugEnabled() bool {
	return globalLogger.level <= LogLevelDebug
}

// formatMessage formats a log message with timestamp and level
func (l *Logger) formatMessage(level string, msg string, fields map[string]interface{}) string {
	timestamp := time.Now().Format("2006-01-02T15:04:05.000Z07:00")
	result := fmt.Sprintf("[%s] %s: %s", timestamp, level, msg)

	if len(fields) > 0 {
		result += " |"
		for k, v := range fields {
			result += fmt.Sprintf(" %s=%v", k, v)
		}
	}

	return result
}

// Debug logs a debug message
func (l *Logger) Debug(msg string, fields ...map[string]interface{}) {
	if l.level <= LogLevelDebug {
		var f map[string]interface{}
		if len(fields) > 0 {
			f = fields[0]
		}
		l.logger.Println(l.formatMessage("DEBUG", msg, f))
	}
}

// Info logs an info message
func (l *Logger) Info(msg string, fields ...map[string]interface{}) {
	if l.level <= LogLevelInfo {
		var f map[string]interface{}
		if len(fields) > 0 {
			f = fields[0]
		}
		l.logger.Println(l.formatMessage("INFO", msg, f))
	}
}

// Warn logs a warning message
func (l *Logger) Warn(msg string, fields ...map[string]interface{}) {
	if l.level <= LogLevelWarn {
		var f map[string]interface{}
		if len(fields) > 0 {
			f = fields[0]
		}
		l.logger.Println(l.formatMessage("WARN", msg, f))
	}
}

// Error logs an error message
func (l *Logger) Error(msg string, fields ...map[string]interface{}) {
	if l.level <= LogLevelError {
		var f map[string]interface{}
		if len(fields) > 0 {
			f = fields[0]
		}
		l.logger.Println(l.formatMessage("ERROR", msg, f))
	}
}

// Global logging functions for convenience
func Debug(msg string, fields ...map[string]interface{}) {
	globalLogger.Debug(msg, fields...)
}

func Info(msg string, fields ...map[string]interface{}) {
	globalLogger.Info(msg, fields...)
}

func Warn(msg string, fields ...map[string]interface{}) {
	globalLogger.Warn(msg, fields...)
}

func LogError(msg string, fields ...map[string]interface{}) {
	globalLogger.Error(msg, fields...)
}

// LogRequest logs a request
func LogRequest(method string, requestID string) {
	Debug("Request received", map[string]interface{}{
		"method":     method,
		"request_id": requestID,
	})
}

// LogResponse logs a response
func LogResponse(method string, requestID string, duration time.Duration, success bool) {
	level := "info"
	if !success {
		level = "error"
	}

	fields := map[string]interface{}{
		"method":      method,
		"request_id":  requestID,
		"duration_ms": duration.Milliseconds(),
		"success":     success,
	}

	if level == "info" {
		Info("Request completed", fields)
	} else {
		LogError("Request failed", fields)
	}
}
