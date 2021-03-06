[![Build status](https://ci.appveyor.com/api/projects/status/virwljswpbuaifb7/branch/master?svg=true)](https://ci.appveyor.com/project/BrianGreen/with-clean-env/branch/master)

# with-clean-env

This is a Windows tool that runs a command in a clean user environment. In particular, the command
will not inherit the current process environment. This is useful to escape the cygwin environment.

## Example

Suppose you want to run `grunt` from a cygwin ssh session. By default, the `PATH` variable for
npm is set as a user variable (not a system variable), so it will not be part of your ssh
environment.

```
$ grunt
-bash: grunt: command not found
```

To avoid this problem, you can use `with-clean-env` to run the command in the default windows
environment for the current user.

```
$ with-clean-env cmd /c grunt
grunt-cli: The grunt command line interface (v1.2.0)
...
```

The original exit code of the process is preserved.

```
$ with-clean-env cmd /c exit /b 123
$ echo %ERRORLEVEL%
123
```

## Usage

```
Usage: with-clean-env cmd [arg1 arg2 ...]

Summary:

    Runs a command with a clean environment. In particular, the
    command does not inherit the current process environment.

For example:

    with-clean-env cmd /c echo hello
```
