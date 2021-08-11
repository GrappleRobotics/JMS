#!/bin/bash

MESSAGE_LOC=../electronics/src/libs/Network/Messages
MESSAGE_TYPE=".proto"
COMP_MESSAGE_LOC="JMS_Messages"

if (($# < 1))
then
	echo "Please specify files in when running"
	echo "./gen.sh nodes.proto nodes2.proto ..."
else
	rm -r $MESSAGE_LOC/$COMP_MESSAGE_LOC
	mkdir -p $MESSAGE_LOC/$COMP_MESSAGE_LOC
	echo "#ifndef JMS_MESSAGES_H" > $MESSAGE_LOC/$COMP_MESSAGE_LOC/messages.h
	echo "#define JMS_MESSAGES_H" >> $MESSAGE_LOC/$COMP_MESSAGE_LOC/messages.h

	for file in "$@"
	do
		MESSAGE_FILE=$file
		echo "Compiling file $MESSAGE_FILE..."

		#Compile .proto files to .ph
		protoc -o$file.pb $file

		python $MESSAGE_LOC/generator/nanopb_generator.py $file.pb
		mv *.h *.c $MESSAGE_LOC/$COMP_MESSAGE_LOC

		COMPILED_MESSAGE="${MESSAGE_FILE//$MESSAGE_TYPE/}.pb.h"

		echo "#include \"$COMPILED_MESSAGE\"" >> $MESSAGE_LOC/$COMP_MESSAGE_LOC/messages.h

		# Create general message file to be included into projects
	done
	echo "#endif" >> $MESSAGE_LOC/$COMP_MESSAGE_LOC/messages.h
	rm -r *.pb
fi