# Vantage6

Create a new Python virtual environment, and activate it:
```shell
$ virtualenv venv
$ source ./venv/bin/activate
```

Install Vantage6 (with the virtual environment activated):
```
$ pip install vantage6
```

Then setup the server
```
$ vserver new --user
? Please enter a configuration-name: myv6
? Enter a human-readable description: MyVantage6
? ip: 0.0.0.0
? Enter port to which the server listens: 5000
? Path of the api: /api
? Database URI: sqlite:///default.sqlite
? Allowed to drop all tables:   True
? Do you want a constant JWT secret?  Yes
? Which level of logging would you like?  DEBUG
[info]  - New configuration created: /home/onno/.config/vantage6/server/myv6.yaml
[info]  - You can start the server by running vserver start --user
```

Import the example fixtures
```
$ vserver import $(realpath ./fixtures.yml) --user --drop-all
```

Start the server
```
$ vserver start --user
```

Build the vantage6 package
```
$ brane build api_spec.yml
```

Test the vantage6 package (using default username/password: admin/password):
```
$ brane test vantage6
✔ The function the execute · login

Please provide input for the chosen function:

✔ username (string) · admin
✔ password (string) · ********

refresh_token:
eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9.eyJpYXQiOjE2MjA4MTQ0MzEsIm5iZiI6MTYyMDgxNDQzMSwianRpIjoiMTUwNWI5YjgtZWM1Ny00YTAzLThjN2MtOGUzNGE1Yjk5ZDA1IiwiaWRlbnRpdHkiOjEsInR5cGUiOiJyZWZyZXNoIn0.1_PI7WrOM84J9tmSrsCmlKMkPUEGd_9PwSrYsgW3mBI

access_token:
eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9.eyJpYXQiOjE2MjA4MTQ0MzEsIm5iZiI6MTYyMDgxNDQzMSwianRpIjoiMjY0OWE3YTQtYzY3YS00NzQ3LWJlNWItZmY4MDZjNTE1MzgxIiwiZXhwIjoxNjIwODM2MDMxLCJpZGVudGl0eSI6MSwiZnJlc2giOmZhbHNlLCJ0eXBlIjoiYWNjZXNzIiwidXNlcl9jbGFpbXMiOnsidHlwZSI6InVzZXIiLCJyb2xlcyI6WyJSb290Il19fQ.C7rAEHP34ZOctdrI_96xmpvO5txVOuPJ9-8Y_4bR18o

user_url:
/api/user/1

refresh_url:
/api/token/refresh
```
