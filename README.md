<p align="center" width="100%">
    <img width="40%" src="https://raw.githubusercontent.com/FlorianKohn/RustOverflow/main/static/logo.svg">
</p>

# RustOverflow
A StackOverflow mockup implemented in Rust using: Diesel, Rocket and Handlebars templates.

The main purpose of the project is to serve as an example for the Rocket ecosystem.
It can be used as a starting point to learn and explore the features that Rocket provides.
The project should in no means considered as complete or production ready.

## Build from Source

To build the project manually, first install the diesel cli using:

`cargo install diesel_cli --no-default-features --features sqlite`

_Note_: Make sure that the SQLite is installed on your system.

Then create a database using:

`diesel migration run --database-url rust_overflow.db3`

And last but not least run the project as usual using:

`cargo run`


## Docker usage
You can also use the provided Dockerfile to deploy the application. To do so first build the image using:

`docker build -t rustoverflow .`

And then start the container:

```
docker run --name rustoverflow 
            -p 8080:80 
            -v <PATH TO DATABSE>:/RustOverflow/rust_overflow.db3 
            rustoverflow
```