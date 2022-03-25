DEST_TARGET_DIR = "strangers"
pkg: build
	mkdir ${DEST_TARGET_DIR} && \
	cp ./target/release/strangers ./${DEST_TARGET_DIR}/ && \
	cp -r ./certs ./${DEST_TARGET_DIR}/ && \
	cp -r ./templates ./${DEST_TARGET_DIR}/ && \
	cp -r ./static ./${DEST_TARGET_DIR}/ && \
	cp ./.env ./${DEST_TARGET_DIR}/.env && \
	cp ./strangers.service ./${DEST_TARGET_DIR}/ && \
	tar zcvf ./${DEST_TARGET_DIR}.tar.gz ./${DEST_TARGET_DIR}
build:
	cargo build --release

clean:
	rm -rf ./${DEST_TARGET_DIR}.tar.gz && \
	rm -rf ./${DEST_TARGET_DIR}
