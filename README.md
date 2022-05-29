<p align="center">
  <img alt="Stump's logo. Description: A young individual sitting on a tree stump reading a book. Inspired by Stump's creator's childhood, where a large amount of his time was spent sitting on a tree stump reading his comic books." src="./.github/images/logo.png" style="width: 50%" />
  <br />
  <a href="https://discord.gg/63Ybb7J3as">
    <img src="https://img.shields.io/discord/972593831172272148?label=Discord&color=5865F2" />
  </a>
  <a href="https://github.com/aaronleopold/stump/blob/main/LICENSE">
    <img src="https://img.shields.io/static/v1?label=License&message=MIT&color=CF9977" />
  </a>
  <!-- <a href="./.github/CHANGELOG.md">
    <img src="https://img.shields.io/github/package-json/v/aaronleopold/stump?logo=azurepipelines&amp;color=0aa8d2" alt="Current Version">
  </a>
  <a href="https://hub.docker.com/r/aaronleopold/stump">
    <img src="https://img.shields.io/docker/pulls/aaronleopold/stump?logo=docker&color=0aa8d2&logoColor=fff" alt="Docker Pulls">
  </a> -->
</p>

Stump is a free and open source comics, manga and digital book server with OPDS support, **heavily** inspired by [Komga](https://github.com/gotson/komga), created with Rust, [Rocket](https://github.com/SergioBenitez/Rocket), [Prisma](https://github.com/Brendonovich/prisma-client-rust) and React.

I love Komga and use it at home, and I thought it would be cool to learn what goes into making something like this myself. I opted to develop this in Rust to hopefully, at the end of all this, create something just as, if not almost as, convenient but with a smaller footprint. _I also just want an excuse to practice Rust!_

<p align='center'>
<img alt="Screenshot of Stump" src="./.github/images/demo-img.png" style="width: 85%" />
</p>

## Roadmap

I'll list the major target features below - I am very open to suggestions and ideas, so feel free to reach out if you have anything you'd like to see!

- Full OPDS + OPDS Page Streaming support
- EPUB, PDF, and CBZ/CBR support
- Customizable server configuration (for both Docker and local hosting)
- Complex but lean and speedy indexing/scanning operations
- Support for a range of metadata operations (e.g. adding/removing tags, changing metadata, etc.)
- Import/export of libraries
- Configurable CBR-to-CBZ conversion
- Small footprint and resource utilization
- Integrated web client (React) served by Rust server

For more, feel free to view the [FAQ](https://stumpapp.dev/faq) page. If you're interested in tracking the development of specific features, you can take a look at the [V1 Project Board](https://github.com/users/aaronleopold/projects/2).

## Getting Started

For information about getting started, how Stump works and manages your library data, and much more detailed information, please visit [stumpapp.dev](https://stumpapp.dev/guides).

## Project Structure

I am ommitting a lot of files and only focusing on the main directories, but the following is the structure of the project:

```bash
.
├── packages
│   ├── core
│   │   ├── frontend
│   │   └── server
│   │       ├── prisma
│   │       ├── prisma-cli
│   │       └── src
│   └── website
├── README.md
└── ...
```

### Core

The core package is where Stump's core functionality is located.

`server`: This is the bulk of Stump's functionality. It is a Rocket server.

`frontend`: The frontend directory is where the web client is located. It is a static React application that is served by Stump.

<!-- This file is used to seed the database with data. -->

### Website

The website package contains a Next.js application for the Stump landing page and documentation pages. The documentation aspect is created using [Markdoc](https://markdoc.io/). This code gets deployed to [stumpapp.dev](http://stumpapp.dev)

## Development Setup

There is now a setup script that handles most of the initial configuration, however ensure you at least have the basics: [pnpm](https://pnpm.io/installation), [rust](https://www.rust-lang.org/tools/install) and [node](https://nodejs.org/en/download/). The script may ask to attempt installing `pnpm` using `npm` if it is not found in your $PATH.

**Ensure you are on the `develop` branch before continuing.**

### Setup Script

If you are on a Windows machine, you'll need to run the following:

```
.\.github\scripts\setup.ps1
```

Otherwise, you can run the following:

```bash
./.github/scripts/setup.sh
```

These scripts will run system checks for `cargo` and `pnpm`, and will install a few additional dependencies, depending on your system. It will then install all the direct, Stump development dependencies, build the frontend bundle (required for server to start), generate the prisma client and sqlite database.

If you face any issues running these, or are using a system that is not supported by the setup scripts, please consider [adding/improving support](https://github.com/aaronleopold/stump/issues) for your system.

### Running the Seed Script

This step isn't required anymore, and will shortly be removed! The built-in client has progressed to a point where it makes more sense to setup your server all through the client. Booting up your server, following the next section's steps, will now prompt you to create the managing account and create your first library.

Leaving this section here for a few weeks to make sure people don't get confused.

## Running Stump

To start the application for development, simply run:

```bash
pnpm core dev
```

This will start both the vite dev server and the rust server, watching for changes. You can also run the server and the frontend in separate processes:

```bash
pnpm core server:dev # start the server
pnpm core frontend:dev # start the frontend
```

## License

Stump codebase is licensed under an [MIT license](./LICENSE). This does not apply to Stump's logo, if you would like to use the logo for anything other than a reference to Stump, please [contact me](aaronleopold1221@gmail.com).

## Contributing

Contributions are very **encouraged** and **welcome**! Please review the [CONTRIBUTING.md](./CONTRIBUTING.md) file beforehand. Thanks!

#### Developer Resources

A few useful resources for developers looking to contribute:

- [Rocket documentation](https://rocket.rs/v0.5-rc/)
- [Prisma documentation](https://prisma.io/docs/prisma-client/introduction)
  - [Prisma Client Rust Documentation](https://github.com/Brendonovich/prisma-client-rust/tree/main/docs)
- [Chakra UI Documentation](https://chakra-ui.com/docs)
- [OPDS specification](https://specs.opds.io/)
- [OPDS Page Streaming](https://vaemendis.net/opds-pse/#:~:text=The%20OPDS%20Page%20Streaming%20Extension,having%20to%20download%20it%20completely.)
- [Getting started with React](https://reactjs.org/docs/getting-started.html)
- [Rust Book](https://doc.rust-lang.org/book/)
