Sensor Net Middleware
=====================

Middleware for the `Sensor Net Project <https://github.com/hannes-hochreiner/sensor-net>`_.

Running
-------

The program gets its configuration from the following environment variables.

:SENSOR_NET_DEVICE: The device name (e.g. "/dev/ttyUSB0")
:SENSOR_NET_KEY: The sensor net encryption key as a hex encoded string
:AUTH0_TENANT: Tenant
:AUTH0_REGION: Region (e.g. "eu")
:AUTH0_CLIENT_ID: Id of the client
:AUTH0_CLIENT_SECRET: Secret token of the client
:AUTH0_CLIENT_AUDIENCE: Audience

Installation
------------

.. code-block:: bash

  cargo install --git https://github.com/hannes-hochreiner/sensor-net-middleware-rs
