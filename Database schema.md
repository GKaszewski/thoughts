# **Thoughts \- Database Schema (PostgreSQL)**

## **1\. Overview**

This document outlines the table structure for the Thoughts platform using PostgreSQL. The design uses UUIDs for primary keys to facilitate decentralization and prevent enumeration attacks. All timestamps are stored with time zones (TIMESTAMPTZ).

## **2\. Schema Diagram (ERD)**

\+-------------+      \+--------------+      \+--------------+  
|    users    |\<--+--|   thoughts   |---+--|\> thought\_tags |  
\+-------------+   |  \+--------------+   |  \+--------------+  
      |           |                       |        ^  
      |           |                       |        |  
      |           |  \+--------------+     |  \+--------------+  
      \+--------+--+--|\>   follows    |\<--+-+--|     tags     |  
      |        |     \+--------------+     |  \+--------------+  
      |        |                          |  
      v        |                          |  
\+-------------+  |                          |  
| top\_friends |\<-+                          |  
\+-------------+                           |  
      |                                     |  
      v                                     |  
\+-------------+                           |  
|  api\_keys   |\<--------------------------+  
\+-------------+

*(Note: Arrows denote foreign key relationships)*

## **3\. Table Definitions**

### **users**

Stores user account and profile information.

| Column Name | Data Type | Constraints | Description |
| :---- | :---- | :---- | :---- |
| id | UUID | PRIMARY KEY, DEFAULT gen\_random\_uuid() | Unique identifier for the user. |
| username | VARCHAR(32) | NOT NULL, UNIQUE | The user's handle. |
| email | VARCHAR(255) | NOT NULL, UNIQUE | The user's email address. |
| password\_hash | TEXT | NOT NULL | Hashed password (using Argon2 or bcrypt). |
| display\_name | VARCHAR(50) | NULL | User's public display name. |
| bio | VARCHAR(160) | NULL | User's public biography. |
| avatar\_url | TEXT | NULL | URL to the user's avatar image. |
| header\_url | TEXT | NULL | URL to the user's header image. |
| custom\_css | TEXT | NULL | User's custom profile CSS. |
| created\_at | TIMESTAMPTZ | NOT NULL, DEFAULT NOW() | Timestamp of account creation. |
| updated\_at | TIMESTAMPTZ | NOT NULL, DEFAULT NOW() | Timestamp of the last profile update. |

### **thoughts**

Stores the content of each post.

| Column Name | Data Type | Constraints | Description |
| :---- | :---- | :---- | :---- |
| id | UUID | PRIMARY KEY, DEFAULT gen\_random\_uuid() | Unique identifier for the thought. |
| user\_id | UUID | NOT NULL, REFERENCES users(id) | The ID of the authoring user. |
| content | VARCHAR(128) | NOT NULL | The text content of the thought. |
| created\_at | TIMESTAMPTZ | NOT NULL, DEFAULT NOW() | Timestamp of when the thought was posted. |

### **follows**

A join table representing the follower/following relationship.

| Column Name | Data Type | Constraints | Description |
| :---- | :---- | :---- | :---- |
| follower\_id | UUID | NOT NULL, REFERENCES users(id) | The user who is initiating the follow. |
| following\_id | UUID | NOT NULL, REFERENCES users(id) | The user who is being followed. |
|  |  | PRIMARY KEY (follower\_id, following\_id) | Ensures a user can't follow someone twice. |

### **top\_friends**

Stores the ordered list of a user's "Top Friends".

| Column Name | Data Type | Constraints | Description |
| :---- | :---- | :---- | :---- |
| user\_id | UUID | NOT NULL, REFERENCES users(id) | The owner of this "Top Friends" list. |
| friend\_id | UUID | NOT NULL, REFERENCES users(id) | The user being displayed as a friend. |
| position | SMALLINT | NOT NULL | The order (1-8) of the friend on the list. |
|  |  | PRIMARY KEY (user\_id, friend\_id) | Ensures a user can't be in the list twice. |
|  |  | UNIQUE (user\_id, position) | Ensures positions are not duplicated. |

### **tags and thought\_tags (for hashtags)**

* **tags**: Stores unique tag names.  
* **thought\_tags**: A join table linking thoughts to tags.

#### **tags**

| Column Name | Data Type | Constraints | Description |
| :---- | :---- | :---- | :---- |
| id | SERIAL | PRIMARY KEY | Unique ID for the tag. |
| name | VARCHAR(50) | NOT NULL, UNIQUE | The tag name (e.g., "welcome"). |

#### **thought\_tags**

| Column Name | Data Type | Constraints | Description |
| :---- | :---- | :---- | :---- |
| thought\_id | UUID | NOT NULL, REFERENCES thoughts(id) | The ID of the thought. |
| tag\_id | INTEGER | NOT NULL, REFERENCES tags(id) | The ID of the tag. |
|  |  | PRIMARY KEY (thought\_id, tag\_id) | Prevents duplicate tags per post. |

### **api\_keys**

Stores hashed API keys for users.

| Column Name | Data Type | Constraints | Description |
| :---- | :---- | :---- | :---- |
| id | UUID | PRIMARY KEY, DEFAULT gen\_random\_uuid() | Unique identifier for the API key. |
| user\_id | UUID | NOT NULL, REFERENCES users(id) | The user who owns this key. |
| key\_hash | TEXT | NOT NULL, UNIQUE | The hashed value of the API key. |
| name | VARCHAR(50) | NOT NULL | A user-provided name for the key. |
| created\_at | TIMESTAMPTZ | NOT NULL, DEFAULT NOW() | Timestamp of when the key was created. |

