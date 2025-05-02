# asus-router

## Compile the code targeting aarch64

```shell
docker run --rm -ti -v `pwd`:/app jpcercal/asus-router
```

## Copying the compiled file to the router

```shell
cp .env.example .env
```

```shell
source .env && SSHPASS="${ASUS_ROUTER_SSH_PWD}" sshpass -e ssh -o StrictHostKeyChecking=no -p ${ASUS_ROUTER_SSH_PORT} "${ASUS_ROUTER_SSH_USER}@${ASUS_ROUTER_SSH_HOST}" "cat > ${ASUS_ROUTER_TARGET_FILE} && chmod +x ${ASUS_ROUTER_TARGET_FILE}" < target/aarch64-unknown-linux-musl/release/asus-router
```
