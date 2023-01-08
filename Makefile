.PHOHY: upload
upload:
	rsync -az --progress -e ssh --delete ~/.cargo/target/doc/ basti@fleark.de:/srv/www/fleark.de/doc
