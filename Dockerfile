FROM alpine
WORKDIR /
COPY tangram .
ENTRYPOINT ["./tangram"]