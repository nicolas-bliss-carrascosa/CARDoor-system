# CARDoor-system
A set of integrated utilities to manage RFID readers and door locks with an intuitive ui.

Components of the Plan:
- Database schema
    - See CARDatabase.sql

- Core Database API 
    - Every interaction with the sensors should be logged
        - Should track time, door id, scanned id
    - Should be easy to lookup scanned id => roles => allowed to open door rn

- Sensor / Lock behavior
    - On Scan Behavior
        - Open encrypted connection with central server
        - Send door id + scanned id
        - Get back open / dont open
        - Comply
    - Ping Central: (every ~1 m)
        - Open connection with central server
        - Send door id
        - Receive instructions (be locked / unlocked)
        - Comply

- UI Database API
    - lmao
    - dont overwrite the logs!!!
    - please dont break public = roleid 0 or else :))

- Failure Modes
    - aren't real :)

- Security??
    - How do signatures work
        - magic with RSA :0
    - How do the sensors talk with the central server
        - Wireguard??