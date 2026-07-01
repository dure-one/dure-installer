/*
 * winhttpd_lib.h - FFI interface for winhttpd
 * Windows HTTP server FFI bindings for Rust
 */

#ifndef WINHTTPD_LIB_H
#define WINHTTPD_LIB_H

#ifdef __cplusplus
extern "C" {
#endif

/**
 * Initialize winhttpd with command-line arguments
 * @param argc Argument count
 * @param argv Argument vector
 * Returns: 0 on success, non-zero on error
 */
int winhttpd_init(int argc, char** argv);

/**
 * Run one iteration of the poll loop
 * Should be called repeatedly while server is running
 */
void winhttpd_poll_once(void);

/**
 * Start the server (sets running flag)
 */
void winhttpd_start(void);

/**
 * Stop the server (clears running flag)
 */
void winhttpd_stop(void);

/**
 * Check if server is running
 * Returns: 1 if running, 0 if stopped
 */
int winhttpd_is_running(void);

/**
 * Cleanup and shutdown winhttpd
 * Should be called after stopping the server
 */
void winhttpd_cleanup(void);

#ifdef __cplusplus
}
#endif

#endif /* WINHTTPD_LIB_H */
