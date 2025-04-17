# glad-web

Source code for the Genetics of Latin American Diversity (GLAD) web portal.
[`paper`]:

Accompanying repositories can be found below:
[`glad-match`]: https://github.com/rlaboulaye/glad-match
[`glad-prep`]: https://github.com/rlaboulaye/glad-prep

## Setup

### Clone this Repository

```shell
$ git clone https://github.com/rlaboulaye/glad-web
$ cd glad-web
```

### Installing Rust and Cargo

Install Rust as described in [The Rust Programming Language, chapter 1](https://doc.rust-lang.org/book/ch01-01-installation.html).

This is the official Rust language manual and is freely available on doc.rust-lang.org.

The latest stable version is fine.


### Installing `sqlx-cli`

SQLx provides a command-line tool for creating and managing databases as well as migrations. It is published
on the Cargo crates registry as `sqlx-cli` and can be installed like so:

```shell
$ cargo install sqlx-cli
```

### Installing `sqlite3`

You can check if `sqlite3` is pre-installed by running the following command:

```shell
$ sqlite3 --version
```

If it is not, you can install it either through your operating system's package manager.

### Configuring the Application

To make working with environment variables easier during development, we can use [.env files] to avoid having
to define the variables every time.

As a starting point, you can simply `cp .env.sample .env` in this repo and modify the `.env` file.

### Setting Up the Application Database

With `sqlx-cli` installed and your `.env` file set up, you only need to run the following command to get the
SQLite database ready for use:

```
$ source .env
$ sqlx db setup
```

### Starting the Application

With everything else set up, all you should have to do at this point is:

```
$ cargo leptos run
```

If successful, the Realworld-compatible API is now listening at port 3000.

