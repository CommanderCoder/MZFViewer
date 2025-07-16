# TWO_TARGETS

A firebase project can have multiple sites.  To deploy the different sites, use deploy targets.  Then the "hosting" attribute will be an array of targets, rather than a single project.

The target name is a local name to refer to the site.

`firebase target:apply hosting sharp mzfviewer`
`firebase target:apply hosting sinclair zxviewer`

In order to have one Firebase Project with two sites, add the "target" attribute.
```
{
  "hosting": [ {
      "target": "sharp",  // "sharp" is the applied TARGET_NAME for the Hosting site "mzfviewer"
      "public": "public",  // contents of this folder are deployed to the site "mzfviewer"

      // ...
    },
    {
      "target": "sinclair",  // "app" is the applied TARGET_NAME for the Hosting site "zxviewer"
      "public": "public",  // contents of this folder are deployed to the site "zxviewer"

      // ...

      "rewrites": [...]  // You can define specific Hosting configurations for each site
    }
  ]
}
```

To view it:

`firebase emulators:start --only hosting:sharp`

`firebase emulators:start --only hosting:sinclair`
