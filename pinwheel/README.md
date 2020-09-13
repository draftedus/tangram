# pinwheel

Pinwheel is Tangram's front end build tool that powers the Tangram app and website. It is a wrapper around [Rollup]() that bundles the TypeScript and CSS.

Here's how it works.

Pinwheel uses filesystem based routing.

```
blog
-
```

You can run `pinwheel dev` to start a development server, and. The server starts instantly because in dev mode each page is compiled as it is requested.

This works for building simple static sites but what if you want to build a dynamic web application?

Instead of
