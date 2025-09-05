# **Thoughts \- API Design (Version 1\)**

## **1\. Overview**

This document specifies the RESTful API for the Thoughts platform.

* **Base URL:** /api/v1  
* **Data Format:** All requests and responses will be in JSON format.  
* **Authentication:** The API uses two primary methods for authentication:  
  1. **JWT (JSON Web Tokens):** For the official web client. The POST /api/v1/auth/login endpoint returns a short-lived JWT. This token must be included in the Authorization: Bearer \<token\> header for all subsequent authenticated requests.  
  2. **API Keys:** For third-party applications. Users can generate long-lived API keys. These keys must be included in the Authorization: ApiKey \<key\> header.

## **2\. API Endpoints**

### **Auth Endpoints**

**POST /auth/register**

* **Description:** Creates a new user account.  
* **Authentication:** Public.  
* **Request Body:**  
  {  
    "username": "frutiger",  
    "email": "aero@example.com",  
    "password": "strongpassword123"  
  }

* **Success Response:** 201 Created with the new User object (password omitted).  
* **Error Responses:** 400 Bad Request (invalid input), 409 Conflict (username or email already exists).

**POST /auth/login**

* **Description:** Authenticates a user and returns a JWT.  
* **Authentication:** Public.  
* **Request Body:**  
  {  
    "username": "frutiger",  
    "password": "strongpassword123"  
  }

* **Success Response:** 200 OK with a JWT.  
  {  
    "token": "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9..."  
  }

* **Error Responses:** 400 Bad Request, 401 Unauthorized.

### **User & Profile Endpoints**

**GET /users/{username}**

* **Description:** Retrieves the public profile of a user.  
* **Authentication:** Public.  
* **Success Response:** 200 OK with a public User object.

**GET /users/me**

* **Description:** Retrieves the full profile of the currently authenticated user (including private details like email).  
* **Authentication:** Required (JWT).  
* **Success Response:** 200 OK with the full User object.

**PUT /users/me**

* **Description:** Updates the profile of the currently authenticated user.  
* **Authentication:** Required (JWT).  
* **Request Body:**  
  {  
    "displayName": "Frutiger Aero Fan",  
    "bio": "Est. 2004",  
    "avatarUrl": "https://...",  
    "headerUrl": "https://...",  
    "customCss": "body { background: blue; }",  
    "topFriends": \["username1", "username2"\]  
  }

* **Success Response:** 200 OK with the updated User object.  
* **Error Responses:** 400 Bad Request.

### **Thoughts (Posts) Endpoints**

**POST /thoughts**

* **Description:** Creates a new thought.  
* **Authentication:** Required (JWT or API Key).  
* **Request Body:**  
  {  
    "content": "This is my first thought\! \#welcome"  
  }

* **Success Response:** 201 Created with the new Thought object.  
* **Error Responses:** 400 Bad Request (e.g., content \> 128 chars).

**GET /users/{username}/thoughts**

* **Description:** Retrieves all thoughts for a specific user, paginated.  
* **Authentication:** Public.  
* **Success Response:** 200 OK with an array of Thought objects.

**DELETE /thoughts/{id}**

* **Description:** Deletes a thought. The user must be the author.  
* **Authentication:** Required (JWT or API Key).  
* **Success Response:** 204 No Content.  
* **Error Responses:** 403 Forbidden, 404 Not Found.

### **Social Endpoints**

**POST /users/{username}/follow**

* **Description:** Follows a user.  
* **Authentication:** Required (JWT).  
* **Success Response:** 204 No Content.  
* **Error Responses:** 404 Not Found, 409 Conflict (already following).

**DELETE /users/{username}/follow**

* **Description:** Unfollows a user.  
* **Authentication:** Required (JWT).  
* **Success Response:** 204 No Content.  
* **Error Responses:** 404 Not Found.

**GET /feed**

* **Description:** Retrieves the main feed for the authenticated user, paginated.  
* **Authentication:** Required (JWT).  
* **Success Response:** 200 OK with an array of Thought objects from followed users.

### **Discovery Endpoints**

**GET /tags/popular**

* **Description:** Retrieves a list of currently popular tags.  
* **Authentication:** Public.  
* **Success Response:** 200 OK with an array of tag strings.

**GET /tags/{tagName}**

* **Description:** Retrieves a feed of all thoughts with a specific tag, paginated.  
* **Authentication:** Public.  
* **Success Response:** 200 OK with an array of Thought objects.

## **3\. Data Models**

**User Object (Public)**

{  
  "username": "frutiger",  
  "displayName": "Frutiger Aero Fan",  
  "bio": "Est. 2004",  
  "avatarUrl": "https://...",  
  "headerUrl": "https://...",  
  "customCss": "body { background: blue; }",  
  "topFriends": \["username1", "username2"\],  
  "joinedAt": "2024-01-01T12:00:00Z"  
}

**Thought Object**

{  
  "id": "uuid-v4-string",  
  "authorUsername": "frutiger",  
  "content": "This is my first thought\! \#welcome",  
  "tags": \["welcome"\],  
  "createdAt": "2024-01-01T12:01:00Z"  
}  
