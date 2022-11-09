
dev:
	make -j 2 srvdev webdev

srvdev:
	cargo run -- -l 127.0.0.1:8400 ./testdir

webdev:
	yarn start

webbuild:
	yarn build