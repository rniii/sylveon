# sylveon

A very simple way to parse CLI arguments.

```rs
sylveon::parse! {
    /// Display hidden files
    all,
    /// Increase output
    verbose+,
    /// List of files to ignore
    ignore?,

    /// List files
    ..path => { /* ... */ }
}
```

[Syntax guide](https://codeberg.org/twink/sylveon/src/branch/main/examples/guide.rs)

## License

Apache-2.0
