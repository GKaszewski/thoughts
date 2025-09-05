"use client";

import { useState, useEffect, FormEvent } from "react";

interface Thought {
  id: number;
  author_id: number;
  content: string;
  created_at: string;
}

export default function Home() {
  // State to store the list of thoughts for the feed
  const [thoughts, setThoughts] = useState<Thought[]>([]);
  // State for the content of the new thought being typed
  const [newThoughtContent, setNewThoughtContent] = useState("");
  // State to manage loading status
  const [isLoading, setIsLoading] = useState(true);
  // State to hold any potential errors during API calls
  const [error, setError] = useState<string | null>(null);

  // Function to fetch the feed from the backend API
  const fetchFeed = async () => {
    try {
      setError(null);
      const response = await fetch("http://localhost:8000/feed");
      if (!response.ok) {
        throw new Error(`HTTP error! status: ${response.status}`);
      }
      const data = await response.json();
      // The API returns { thoughts: [...] }, so we access the nested array
      setThoughts(data.thoughts || []);
    } catch (e: unknown) {
      console.error("Failed to fetch feed:", e);
      setError(
        "Could not load the feed. The backend might be busy. Please try refreshing."
      );
    } finally {
      setIsLoading(false);
    }
  };

  // useEffect hook to fetch the feed when the component first loads
  useEffect(() => {
    fetchFeed();
  }, []);

  // Handler for submitting the new thought form
  const handleSubmitThought = async (e: FormEvent) => {
    e.preventDefault();
    if (!newThoughtContent.trim()) return; // Prevent empty posts

    try {
      const response = await fetch("http://localhost:8000/thoughts", {
        method: "POST",
        headers: {
          "Content-Type": "application/json",
        },
        // We are hardcoding author_id: 1 as we don't have auth yet
        body: JSON.stringify({ content: newThoughtContent, author_id: 1 }),
      });

      if (!response.ok) {
        throw new Error(`HTTP error! status: ${response.status}`);
      }

      // Clear the input box
      setNewThoughtContent("");
      // Refresh the feed to show the new post
      fetchFeed();
    } catch (e: unknown) {
      console.error("Failed to post thought:", e);
      setError("Failed to post your thought. Please try again.");
    }
  };

  return (
    <div className="font-sans bg-gradient-to-br from-sky-200 via-teal-100 to-green-200 min-h-screen text-gray-800">
      <div className="container mx-auto max-w-2xl p-4 sm:p-6">
        {/* Header */}
        <header className="text-center my-6">
          <h1
            className="text-5xl font-bold text-white"
            style={{ textShadow: "2px 2px 4px rgba(0,0,0,0.2)" }}
          >
            Thoughts
          </h1>
          <p className="text-white/80 mt-2">
            Your space on the decentralized web.
          </p>
        </header>

        {/* New Thought Form */}
        <div className="bg-white/70 backdrop-blur-lg rounded-xl shadow-lg p-5 mb-8">
          <form onSubmit={handleSubmitThought}>
            <textarea
              value={newThoughtContent}
              onChange={(e) => setNewThoughtContent(e.target.value)}
              className="w-full h-24 p-3 rounded-lg border border-gray-300 focus:ring-2 focus:ring-sky-400 focus:outline-none resize-none transition-shadow"
              placeholder="What's on your mind?"
              maxLength={128}
            />
            <div className="flex justify-between items-center mt-3">
              <span className="text-sm text-gray-500">
                {128 - newThoughtContent.length} characters remaining
              </span>
              <button
                type="submit"
                className="px-6 py-2 bg-sky-500 text-white font-semibold rounded-full shadow-md hover:bg-sky-600 active:scale-95 transition-all duration-150 ease-in-out disabled:bg-gray-400"
                disabled={!newThoughtContent.trim()}
              >
                Post
              </button>
            </div>
          </form>
        </div>

        {/* Feed Section */}
        <main>
          {isLoading ? (
            <p className="text-center text-gray-600">Loading feed...</p>
          ) : error ? (
            <div className="bg-red-100 border border-red-400 text-red-700 px-4 py-3 rounded-lg text-center">
              <p>{error}</p>
            </div>
          ) : thoughts.length === 0 ? (
            <p className="text-center text-gray-600">
              The feed is empty. Follow some users to see their thoughts!
            </p>
          ) : (
            <div className="space-y-4">
              {thoughts.map((thought) => (
                <div
                  key={thought.id}
                  className="bg-white/80 backdrop-blur-lg rounded-xl shadow-lg p-4 transition-transform hover:scale-[1.02]"
                >
                  <div className="flex items-center mb-2">
                    <div className="w-10 h-10 rounded-full bg-gradient-to-br from-green-300 to-sky-400 flex items-center justify-center font-bold text-white mr-3">
                      {/* Placeholder for avatar */}
                      {thought.author_id}
                    </div>
                    <div>
                      <p className="font-bold">User {thought.author_id}</p>
                      <p className="text-xs text-gray-500">
                        {new Date(thought.created_at).toLocaleString()}
                      </p>
                    </div>
                  </div>
                  <p className="text-gray-800 break-words">{thought.content}</p>
                </div>
              ))}
            </div>
          )}
        </main>
      </div>
    </div>
  );
}
