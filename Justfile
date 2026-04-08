default: build

build *ARGS:
    cargo build {{ARGS}}

build-image LABEL:
    podman build -t comingle:{{LABEL}} .

save-image LABEL: (build-image LABEL)
    mkdir -p dist
    rm -f dist/comingle-{{LABEL}}.tar
    podman save comingle:{{LABEL}} -o dist/comingle-{{LABEL}}.tar
