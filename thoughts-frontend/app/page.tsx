"use client";

import { useState, useEffect, FormEvent } from "react";
import { Button } from "@/components/ui/button";
import { Textarea } from "@/components/ui/textarea";
import { Card, CardHeader, CardContent } from "@/components/ui/card";
import { Alert } from "@/components/ui/alert";

interface Thought {
  id: number;
  author_id: number;
  content: string;
  created_at: string;
}

export default function Home() {
  const [thoughts, setThoughts] = useState<Thought[]>([]);
  const [newThoughtContent, setNewThoughtContent] = useState("");
  const [isLoading, setIsLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  const fetchFeed = async () => {
    try {
      setError(null);
      const response = await fetch("http://localhost:8000/feed");
      if (!response.ok) {
        throw new Error(`HTTP error! status: ${response.status}`);
      }
      const data = await response.json();
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

  useEffect(() => {
    fetchFeed();
  }, []);

  const handleSubmitThought = async (e: FormEvent) => {
    e.preventDefault();
    if (!newThoughtContent.trim()) return;

    try {
      const response = await fetch("http://localhost:8000/thoughts", {
        method: "POST",
        headers: {
          "Content-Type": "application/json",
        },
        body: JSON.stringify({ content: newThoughtContent, author_id: 1 }),
      });

      if (!response.ok) {
        throw new Error(`HTTP error! status: ${response.status}`);
      }

      setNewThoughtContent("");
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
        <Card className="bg-white/70 backdrop-blur-lg rounded-xl shadow-lg p-5 mb-8">
          <form onSubmit={handleSubmitThought}>
            <Textarea
              value={newThoughtContent}
              onChange={(e: React.ChangeEvent<HTMLTextAreaElement>) =>
                setNewThoughtContent(e.target.value)
              }
              className="resize-none"
              placeholder="What's on your mind?"
              maxLength={128}
            />
            <div className="flex justify-between items-center mt-3">
              <span className="text-sm text-gray-500">
                {128 - newThoughtContent.length} characters remaining
              </span>
              <Button
                type="submit"
                variant="default"
                disabled={!newThoughtContent.trim()}
              >
                Post
              </Button>
            </div>
          </form>
        </Card>

        {/* Feed Section */}
        <main>
          {isLoading ? (
            <p className="text-center text-gray-600">Loading feed...</p>
          ) : error ? (
            <Alert variant="destructive" className="text-center">
              {error}
            </Alert>
          ) : thoughts.length === 0 ? (
            <p className="text-center text-gray-600">
              The feed is empty. Follow some users to see their thoughts!
            </p>
          ) : (
            <div className="space-y-4">
              {thoughts.map((thought) => (
                <Card
                  key={thought.id}
                  className="bg-white/80 backdrop-blur-lg rounded-xl shadow-lg p-4 transition-transform hover:scale-[1.02]"
                >
                  <CardHeader className="flex items-center mb-2">
                    <div className="w-10 h-10 rounded-full bg-gradient-to-br from-green-300 to-sky-400 flex items-center justify-center font-bold text-white mr-3">
                      {thought.author_id}
                    </div>
                    <div>
                      <p className="font-bold">User {thought.author_id}</p>
                      <p className="text-xs text-gray-500">
                        {new Date(thought.created_at).toLocaleString()}
                      </p>
                    </div>
                  </CardHeader>
                  <CardContent>
                    <p className="text-gray-800 break-words">
                      {thought.content}
                    </p>
                  </CardContent>
                </Card>
              ))}
            </div>
          )}
        </main>
      </div>
    </div>
  );
}
