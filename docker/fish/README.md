

# Fish Docker Image

This Dockerfile builds an image with fish, tabry, and one example tabry file installed, for testing purposes.

To test with this docker image, run these commands from the root project folder:

```sh
docker build -t fish-tabry -f docker/fish/Dockerfile .
docker run --rm -it fish-tabry

> tabry fish /tabry/ | source
> tabry_completion_init "foo"
foo <<tab>> # shows completions
```