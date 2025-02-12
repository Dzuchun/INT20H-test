## Backend Functionality Overview

Currently, some backend functionality may not be visible due to the lack of corresponding frontend implementation.

### Terminology Used in the Project:

- **Quest**: A set of pages along with quest-related information (title, description).
- **Page**: A collection of content on the page and associated questions.

### API Capabilities:

- **User Management**:
    - Registration
    - Authorization
    - Retrieving general user information
    - Updating avatar photo
    - Fetching avatar

- **Quest Management**:
    - Creating a quest
    - Updating/retrieving its internal information and pages
    - Fetching a list of quests created by the sender
    - Joining a quest
    - **Publishing** created by the sender quest (making it publicly readable and locking it from further modifications)

- **Quest Interaction**:
    - Retrieving its internal information
    - Fetching a list of quests the sender is/was participating in
    - Submitting a rating and comment for a quest (only after completing it)
    - Fetching author ratings based on the average score of all their quests

### Missing Functionality:

The ability to **complete a quest using WebSockets** (submitting answers to page questions) has not yet been
implemented.

# Start

After cloning project run these commands

- ``docker build -t backend_server_ferebrum .``
- ``docker run -p 80:80 backend_server_ferebrum``

In such way webserver will be started so you can check api, also it will show in site root existing frontend part

Changes will not be saved between docker run`s