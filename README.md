# Moved to https://codeberg.org/icewind/vdf-reader

# vdf-reader

A parser for Valve's Data Format v1 (VDF) also known as [KeyValues](https://developer.valvesoftware.com/wiki/KeyValues).

The parser focuses on being able to deal with all the various weird forms vdf takes in the wild and providing access to the data stream instead of always requiring parsing the file in full.

## Serde

This crate implements a deserializer for serde, but because VDF doesn't map that well only the serde data model not every type might deserialize properly.

### Limitations

- Because the boolean values `0` and `1` can't be distinguished from numbers, it is not possible to use booleans in untagged enums.
- When deserializing arrays by settings the same key multiple times, the keys have to be consecutive.

  ```vdf
  key: 1
  key: 2
  other: 3
  ```

  will work, but

  ```vdf
  key: 1
  other: 3
  key: 2
  ```

  will not.

### Tagged enum root

To help deserialize some common vdf formats, you can use a tagged enum as the root element instead of a struct.

```vdf
"Variant1" {
    content 1
}
```

or

```vdf
"Variant2" {
    other foo
}
```

can be deserialized into a

```rust
enum Data {
    Variant1 {
        content: bool,
    },
    Variant2 {
        other: String,
    }
}
```
