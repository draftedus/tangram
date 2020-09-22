<p align="center">
	<img src="pinwheel.svg" title="Pinwheel">
</p>

# Pinwheel

Pinwheel is Tangram's front end build tool that powers the Tangram app and website.

When we started work on Tangram, we had a number of requirements/desirements to balance:

1. We wanted to use the same code for user interface elements across both the app and website. This would give us a consistent look and feel and allow us to embed subcomponents of the app in the website.

2. We wanted fast incremental compile times in development. Web development involves a lot of tweaking. It is really frustrating to wait even a few seconds after each minor change before seeing it in your browser.

3. We wanted simple deployment.

Here's how it works.

Pinwheel uses filesystem based routing like [Next.js](), except that there is one more level of hierarchy.

```
example
└── pages
   ├── index
   │  ├── client.tsx
   │  └── static.tsx
   └── posts
      └── 2020-01-01-hello-world
         └── static.tsx
```

You can run `pinwheel dev` to start a development server, and. The server starts instantly because in dev mode each page is compiled as it is requested.

This works for building simple static sites but what if you want to build a dynamic web application?

Instead of
