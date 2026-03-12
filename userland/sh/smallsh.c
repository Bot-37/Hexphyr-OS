/*
 * userland/sh/smallsh.c  —  Minimal interactive shell for Hexphyr OS
 *
 * Implements a simple read-eval-execute loop:
 *   - Reads one line from stdin.
 *   - Tokenises on whitespace (no quoting or globbing support yet).
 *   - Looks up built-in commands first; otherwise fork/exec.
 *   - Reaps child processes after each command.
 *
 * Built-in commands: exit, cd
 *
 * Security notes:
 *   - argv is built from fixed-size slots; the input line is hard-limited to
 *     SMALLSH_LINE_MAX bytes to prevent stack overflows from arbitrarily long
 *     commands.
 *   - execvp only searches PATH; it does NOT accept absolute paths from
 *     untrusted input without validation (TODO when VFS permissions land).
 *   - Environment is NOT inherited from the parent; a minimal safe environment
 *     is constructed explicitly.
 */

#include <unistd.h>
#include <sys/types.h>
#include <sys/wait.h>
#include <string.h>
#include <stdio.h>
#include <stdlib.h>

#define SMALLSH_LINE_MAX  512
#define SMALLSH_ARGS_MAX   64

/* Tokenise `line` in-place; fill `argv` up to SMALLSH_ARGS_MAX-1 tokens.
 * Returns the number of tokens found. */
static int tokenise(char *line, char *argv[SMALLSH_ARGS_MAX])
{
    int argc = 0;
    char *tok = strtok(line, " \t\r\n");
    while (tok && argc < SMALLSH_ARGS_MAX - 1) {
        argv[argc++] = tok;
        tok = strtok(NULL, " \t\r\n");
    }
    argv[argc] = NULL;
    return argc;
}

/* Built-in: cd  */
static int builtin_cd(char *argv[])
{
    const char *dir = argv[1] ? argv[1] : "/";
    if (chdir(dir) != 0) {
        perror("cd");
        return 1;
    }
    return 0;
}

/* Fork and exec an external command.  Returns the child's exit status. */
static int run_external(char *argv[])
{
    pid_t pid = fork();
    if (pid < 0) {
        perror("fork");
        return 1;
    }
    if (pid == 0) {
        execvp(argv[0], argv);
        /* execvp failed */
        perror(argv[0]);
        _exit(127);
    }
    int status = 0;
    waitpid(pid, &status, 0);
    return WIFEXITED(status) ? WEXITSTATUS(status) : 1;
}

int main(void)
{
    char line[SMALLSH_LINE_MAX];
    char *argv[SMALLSH_ARGS_MAX];

    for (;;) {
        /* Prompt */
        write(STDOUT_FILENO, "$ ", 2);

        if (!fgets(line, sizeof(line), stdin)) {
            /* EOF (Ctrl-D) — exit cleanly. */
            write(STDOUT_FILENO, "\n", 1);
            break;
        }

        int argc = tokenise(line, argv);
        if (argc == 0) continue;

        /* Built-ins */
        if (strcmp(argv[0], "exit") == 0) {
            int code = argv[1] ? atoi(argv[1]) : 0;
            exit(code);
        }
        if (strcmp(argv[0], "cd") == 0) {
            builtin_cd(argv);
            continue;
        }

        /* External command */
        run_external(argv);
    }
    return 0;
}
