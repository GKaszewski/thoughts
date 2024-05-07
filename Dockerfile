FROM python:3.12-slim-buster AS builder

WORKDIR /usr/src/thoughts

ENV PYTHONDONTWRITEBYTECODE 1
ENV PYTHONUNBUFFERED 1

RUN apt-get update && \
    apt-get install -y --no-install-recommends gcc

# install requirements
COPY requirements.txt .
RUN pip wheel --no-cache-dir --no-deps --wheel-dir /usr/src/szwindel/wheels -r requirements.txt

FROM python:3.12-slim-buster
RUN mkdir -p /home/thoughts

RUN addgroup --system szwindel && adduser --system --group szwindel

# create the appropriate directories
ENV HOME=/home/thoughts
ENV APP_HOME=/home/thoughts/web
RUN mkdir $APP_HOME
RUN mkdir $APP_HOME/static
RUN mkdir $APP_HOME/media
WORKDIR $APP_HOME

# install dependencies
RUN apt-get update && apt-get install -y --no-install-recommends netcat binutils libproj-dev
COPY --from=builder /usr/src/thoughts/wheels /wheels
COPY --from=builder /usr/src/thoughts/requirements.txt .
RUN pip install --upgrade pip
RUN pip install --no-cache /wheels/*

# copy entrypoint.prod.sh
COPY entrypoint.sh $HOME/entrypoint_moved.sh
RUN sed -i 's/\r$//g'  $HOME/entrypoint_moved.sh
RUN chmod +x  $HOME/entrypoint_moved.sh

COPY . $APP_HOME
RUN chown -R thoughts:thoughts $APP_HOME
USER thoughts

ENTRYPOINT ["/home/thoughts/entrypoint_moved.sh"]