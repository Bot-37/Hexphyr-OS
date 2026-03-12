/*
 * userland/init/init.c  —  PID-1 init process
 *
 * This is the first userland process started by the Hexphyr kernel.  Its
 * responsibilities are:
 *   1. Open standard file descriptors (stdin/stdout/stderr) once the VFS
 *      and device layer are available.
 *   2. Start the default shell (smallsh).
 *   3. Reap zombie children in an infinite wait loop so that the process
 *      table does not fill up.
 *
 * Current status: skeleton suitable for compilation once the kernel provides
 * a minimal POSIX-compatible syscall ABI.
 *
 * Security notes:
 *   - init never calls exec() directly with user-supplied strings.
 *   - All child processes are spawned with a minimal environment.
 *   - Unhandled signals default to SIG_DFL (typically terminate/ignore),
 *     handled explicitly in the signal setup below.
 */

#include <unistd.h>
#include <sys/types.h>
#include <sys/wait.h>
#include <signal.h>
#include <stdio.h>
#include <stdlib.h>

/* Path to the default shell binary on the Hexphyr VFS. */
#define DEFAULT_SHELL  "/bin/sh"

/* Minimal safe environment passed to child processes. */
static char *const safe_environ[] = {
    "PATH=/bin:/usr/bin",
    "HOME=/",
    "TERM=vt100",
    NULL
};

/* Spawn the default shell and return its PID, or -1 on error. */
static pid_t spawn_shell(void)
{
    pid_t pid = fork();
    if (pid < 0) {
        /* fork() failed — the kernel may not yet support it. */
        write(STDERR_FILENO, "init: fork() failed\n", 20);
        return -1;
    }
    if (pid == 0) {
        /* Child: exec the shell. */
        char *argv[] = { DEFAULT_SHELL, NULL };
        execvpe(DEFAULT_SHELL, argv, safe_environ);
        /* If we reach here, execvpe() failed. */
        write(STDERR_FILENO, "init: exec failed\n", 18);
        _exit(127);
    }
    return pid;
}

int main(void)
{
    /* PID 1 should ignore SIGINT/SIGQUIT; children inherit SIG_DFL. */
    signal(SIGINT,  SIG_IGN);
    signal(SIGQUIT, SIG_IGN);
    signal(SIGTERM, SIG_IGN);

    /* Start the interactive shell. */
    pid_t shell_pid = spawn_shell();

    /* Zombie-reaping loop: wait for any child to exit and restart the
     * shell if it terminates. */
    for (;;) {
        int   status = 0;
        pid_t child  = wait(&status);
        if (child < 0) {
            /* wait() returned an error (no children or interrupted). */
            continue;
        }
        /* If the shell exited, restart it. */
        if (child == shell_pid) {
            shell_pid = spawn_shell();
        }
    }
    /* Unreachable. */
    return 0;
}
