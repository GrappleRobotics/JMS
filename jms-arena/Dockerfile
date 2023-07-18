# To be invoked in project root
FROM jms-base:latest

WORKDIR /work

ADD jms-arena /work/jms-arena
RUN cd /work/jms-arena && cargo build --release

CMD /work/jms-arena/target/release/jms-arena