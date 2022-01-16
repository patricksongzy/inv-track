FROM node:16
WORKDIR /usr/src/app/app-ui
COPY ./package*.json ./
RUN npm ci
EXPOSE ${PORT}