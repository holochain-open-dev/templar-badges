# Templar badges

Holochain zome to issue public badges to anyone following these rules:

* Anyone can create a new badge, setting the necessary number of claims to achieve the badge
* Only people with a badge can issue it to other people. (Exception: the badge creator can always issue that badge)

## Todo list

* [ ] Refactor code not to use `update_entry`
* [ ] Implement UI native module
* [ ] Publish code to `npm` and `crates.io`

## Building

Assuming you have [nix-shell](https://developer.holochain.org/docs/install/) installed, to build the DNA, execute:

```bash
nix-shell
cd dna
hc package
```
