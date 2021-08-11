#!/bin/bash

PROTO_LOCATION="../../../../../protos"
MESSAGE_LOC=./JMS_Messages
MESSAGE_TYPE=".proto"

rm -r $MESSAGE_LOC
mkdir -p $MESSAGE_LOC

echo "#ifndef JMS_MESSAGES_H" > $MESSAGE_LOC/messages.h
echo "#define JMS_MESSAGES_H" >> $MESSAGE_LOC/messages.h

cd $PROTO_LOCATION
protoFiles=`echo *.proto`
cd -
for file in $protoFiles
do
	echo $file
	cp $PROTO_LOCATION/$file ./


	#Compile .proto files to .ph
	echo "Compiling file $file..."
	protoc -o$file.pb $file

	python ./generator/nanopb_generator.py $file.pb
	COMPILED_MESSAGE="${file//$MESSAGE_TYPE/}.pb"
	mv $COMPILED_MESSAGE.c $COMPILED_MESSAGE.h $MESSAGE_LOC/

	echo "#include \"$COMPILED_MESSAGE.h\"" >> $MESSAGE_LOC/messages.h
done
echo "#endif" >> $MESSAGE_LOC/$COMP_MESSAGE_LOC/messages.h
rm -r *.pb *.proto