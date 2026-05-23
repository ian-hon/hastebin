"""
Example: AI Assistant Using Hastebin API in Conversations
This demonstrates how an AI agent would interact with Hastebin when users share paste URLs
"""

import requests
import json
from typing import Optional, List, Dict
from urllib.parse import urlparse

class HastebinAI:
    """AI-friendly wrapper for Hastebin API - Conversational Use Cases"""
    
    def __init__(self, base_url: str = "https://backend.ianhon.com/hastebin", frontend_url: str = "https://hastebin.ianhon.com"):
        self.base_url = base_url
        self.frontend_url = frontend_url
    
    @staticmethod
    def extract_id_from_url(url: str) -> Optional[int]:
        """
        Extract paste ID from a hastebin URL and convert to decimal
        
        Examples:
            https://hastebin.ianhon.com/abc123 -> 11259375
            https://hastebin.ianhon.com/abc123.js -> 11259375
            hastebin.ianhon.com/abc -> 2748
        """
        try:
            # Handle URLs with or without protocol
            if not url.startswith(('http://', 'https://')):
                url = 'https://' + url
            
            parsed = urlparse(url)
            path = parsed.path.lstrip('/')
            
            # Remove file extension if present (e.g., .js, .py)
            hex_id = path.split('.')[0]
            
            # Convert from hex to decimal
            decimal_id = int(hex_id, 16)
            return decimal_id
        except (ValueError, AttributeError):
            return None
    
    @staticmethod
    def id_to_share_url(decimal_id: int, frontend_url: str = "https://hastebin.ianhon.com") -> str:
        """Convert a decimal paste ID to a shareable URL"""
        hex_id = hex(decimal_id)[2:]  # Remove '0x' prefix
        return f"{frontend_url}/{hex_id}"
    
    def fetch_paste_from_url(self, url: str) -> Optional[dict]:
        """
        Fetch a paste from a user-shared URL
        
        This is the PRIMARY method for conversational AI - when a user shares a link,
        use this to fetch and analyze the content.
        """
        paste_id = self.extract_id_from_url(url)
        if not paste_id:
            raise ValueError(f"Could not extract valid paste ID from URL: {url}")
        
        return self.fetch_paste(paste_id)
        """Fetch OpenAPI specification to understand available endpoints"""
        response = requests.get(f"{self.base_url}/openapi.json")
        return response.json()
    
    def get_ai_instructions(self) -> str:
        """Fetch human-readable instructions for AI agents"""
        response = requests.get(f"{self.base_url}/ai-instructions")
        return response.text
    
    def create_paste(
        self,
        content: str,
        title: Optional[str] = None,
        author: Optional[str] = None,
        comments_enabled: bool = True,
        signature: Optional[str] = None,
        expires_in_hours: Optional[int] = None,
        fork_from: Optional[int] = None
    ) -> dict:
        """Create a new paste"""
        import time
        
        payload = {
            "content": content,
            "title": title,
            "author": author,
            "comments_enabled": comments_enabled,
            "checksum_passphrase": signature,
            "forked_from": fork_from
        }
        
        if expires_in_hours:
            expires_at = int(time.time() + expires_in_hours * 3600)
            payload["expires_at"] = expires_at
        
        response = requests.post(f"{self.base_url}/paste/create", json=payload)
        response.raise_for_status()
        result = response.json()
        
        # Add frontend URL for sharing
        paste_id = result["id"]
        result["frontend_url"] = self.id_to_share_url(paste_id, self.frontend_url)
        
        return result
    
    def create_multifile_paste(
        self,
        files: List[Dict[str, str]],
        title: Optional[str] = None,
        **kwargs
    ) -> dict:
        """Create a paste with multiple files"""
        content = json.dumps([
            {"fileName": f["name"], "content": f["content"]}
            for f in files
        ])
        return self.create_paste(content=content, title=title, **kwargs)
    
    def fetch_paste(self, paste_id: int) -> dict:
        """Fetch a paste by ID"""
        response = requests.get(f"{self.base_url}/paste/fetch/{paste_id}")
        response.raise_for_status()
        return response.json()
    
    def add_comment(
        self,
        paste_id: int,
        content: str,
        from_row: int,
        to_row: int,
        from_column: int = 0,
        to_column: int = 0,
        page_index: int = 0,
        author: Optional[str] = None
    ) -> dict:
        """Add a comment to a specific code selection"""
        payload = {
            "paste_id": paste_id,
            "content": content,
            "author": author,
            "page_index": page_index,
            "from_row": from_row,
            "from_column": from_column,
            "to_row": to_row,
            "to_column": to_column
        }
        
        response = requests.post(f"{self.base_url}/comment/create", json=payload)
        response.raise_for_status()
        return response.json()
    
    def get_comments(self, paste_id: int) -> List[dict]:
        """Get all comments for a paste"""
        response = requests.get(f"{self.base_url}/comment/paste/{paste_id}")
        response.raise_for_status()
        return response.json()


# Conversational Example Scenarios

def scenario_user_shares_link():
    """
    SCENARIO 1: User asks about their code
    User: "Can you explain this code? https://hastebin.ianhon.com/abc123"
    """
    ai = HastebinAI()
    
    user_message = "Can you explain this code? https://hastebin.ianhon.com/abc123"
    url = "https://hastebin.ianhon.com/abc123"
    
    print(f"👤 User: {user_message}\n")
    
    try:
        # Fetch the paste from the URL the user shared
        paste_data = ai.fetch_paste_from_url(url)
        
        paste = paste_data["paste"]
        code = paste["content"]
        
        print(f"🤖 AI Assistant:")
        print(f"   I've fetched your code from paste #{paste['id']}")
        print(f"   Title: {paste.get('title', 'Untitled')}")
        print(f"   Lines of code: {len(code.splitlines())}")
        print(f"\n   Code analysis:")
        print(f"   {code[:200]}..." if len(code) > 200 else f"   {code}")
        print(f"\n   [... AI would analyze and explain the code here ...]")
        
        return paste_data
    except Exception as e:
        print(f"❌ Error fetching paste: {e}")
        return None


def scenario_improve_code():
    """
    SCENARIO 2: User asks for code improvements
    User: "Can you improve this code? https://hastebin.ianhon.com/def456"
    """
    ai = HastebinAI()
    
    user_url = "https://hastebin.ianhon.com/def456"
    print(f"👤 User: Can you improve this code? {user_url}\n")
    
    try:
        # 1. Fetch original paste
        original = ai.fetch_paste_from_url(user_url)
        original_paste = original["paste"]
        original_code = original_paste["content"]
        
        print(f"🤖 AI Assistant: Let me analyze and improve your code...")
        print(f"   Original paste: {original_paste.get('title', 'Untitled')}\n")
        
        # 2. Simulate code improvement (in reality, AI would analyze and improve)
        improved_code = f"""# Improved version with better practices

{original_code}

# Added error handling and documentation
"""
        
        # 3. Create a fork with improvements
        fork = ai.create_paste(
            content=improved_code,
            title=f"Improved: {original_paste.get('title', 'Code')}",
            author="AI Assistant",
            comments_enabled=True,
            forked_from=original_paste["id"]
        )
        
        print(f"✅ I've created an improved version:")
        print(f"   {fork['frontend_url']}")
        print(f"\n   Key improvements:")
        print(f"   • Added error handling")
        print(f"   • Improved documentation")
        print(f"   • Better code structure")
        print(f"\n   The fork is linked to your original, so you can see the diff!")
        
        return fork
    except Exception as e:
        print(f"❌ Error: {e}")
        return None


def scenario_add_review_comments():
    """
    SCENARIO 3: User asks for code review
    User: "Can you review this and add suggestions? https://hastebin.ianhon.com/review123"
    """
    ai = HastebinAI()
    
    user_url = "https://hastebin.ianhon.com/review123"
    print(f"👤 User: Can you review this and add suggestions? {user_url}\n")
    
    try:
        # Fetch the paste
        paste_data = ai.fetch_paste_from_url(user_url)
        paste = paste_data["paste"]
        
        print(f"🤖 AI Assistant: Reviewing your code...")
        
        # Check if comments are enabled
        if not paste["comments_enabled"]:
            print(f"   ⚠️  Comments aren't enabled on this paste.")
            print(f"   Would you like me to create a fork with comments enabled?")
            return None
        
        # Add review comments on specific sections
        comments = [
            {
                "content": "Consider using async/await here for better readability and error handling",
                "from_row": 10,
                "to_row": 15
            },
            {
                "content": "This could be refactored into a separate function for reusability",
                "from_row": 20,
                "to_row": 25
            },
            {
                "content": "Great use of list comprehension! Very Pythonic.",
                "from_row": 30,
                "to_row": 30
            }
        ]
        
        print(f"\n   Adding {len(comments)} review comments...")
        
        for i, comment in enumerate(comments, 1):
            ai.add_comment(
                paste_id=paste["id"],
                content=comment["content"],
                from_row=comment["from_row"],
                to_row=comment["to_row"],
                author="AI Code Reviewer"
            )
            print(f"   💬 Comment {i}: Lines {comment['from_row']}-{comment['to_row']}")
            print(f"      {comment['content'][:60]}...")
        
        print(f"\n✅ Review complete! View the comments on your paste:")
        print(f"   {user_url}")
        
    except Exception as e:
        print(f"❌ Error: {e}")


def scenario_read_existing_comments():
    """
    SCENARIO 4: User asks about existing feedback
    User: "What do people think about my code? https://hastebin.ianhon.com/feedback789"
    """
    ai = HastebinAI()
    
    user_url = "https://hastebin.ianhon.com/feedback789"
    print(f"👤 User: What do people think about my code? {user_url}\n")
    
    try:
        # Extract paste ID
        paste_id = ai.extract_id_from_url(user_url)
        
        # Fetch paste and comments
        paste_data = ai.fetch_paste_from_url(user_url)
        comments = ai.get_comments(paste_id)
        
        print(f"🤖 AI Assistant: Let me check the feedback on your paste...")
        
        if not comments:
            print(f"   No comments yet. Would you like me to review it?")
            return
        
        print(f"\n   Found {len(comments)} comments:\n")
        
        for i, comment in enumerate(comments, 1):
            author = comment.get("author", "Anonymous")
            content = comment["content"]
            lines = f"{comment['from_row']}-{comment['to_row']}"
            
            print(f"   💬 Comment {i} (by {author}):")
            print(f"      Lines {lines}: {content}")
        
        print(f"\n   Overall, the feedback is mostly positive with some suggestions for improvement.")
        
    except Exception as e:
        print(f"❌ Error: {e}")


def scenario_multifile_sharing():
    """
    SCENARIO 5: AI creates multi-file project for user
    User: "Can you create a simple React component with styles?"
    """
    ai = HastebinAI()
    
    print(f"👤 User: Can you create a simple React component with styles?\n")
    
    files = [
        {
            "name": "Button.jsx",
            "content": """import React from 'react';
import './Button.css';

export const Button = ({ children, onClick }) => {
  return (
    <button className="custom-button" onClick={onClick}>
      {children}
    </button>
  );
};"""
        },
        {
            "name": "Button.css",
            "content": """.custom-button {
  padding: 10px 20px;
  background-color: #007bff;
  color: white;
  border: none;
  border-radius: 5px;
  cursor: pointer;
}

.custom-button:hover {
  background-color: #0056b3;
}"""
        },
        {
            "name": "App.jsx",
            "content": """import React from 'react';
import { Button } from './Button';

function App() {
  return (
    <div>
      <Button onClick={() => alert('Clicked!')}>
        Click Me
      </Button>
    </div>
  );
}

export default App;"""
        }
    ]
    
    print(f"🤖 AI Assistant: Creating a React button component...")
    
    result = ai.create_multifile_paste(
        files=files,
        title="React Button Component",
        author="AI Assistant",
        comments_enabled=True
    )
    
    print(f"\n✅ Created a {len(files)}-file project:")
    print(f"   {result['frontend_url']}")
    print(f"\n   Files included:")
    for f in files:
        print(f"   • {f['name']}")
    print(f"\n   You can view, copy, or fork this component!
    return result


if __name__ == "__main__":
    print("🤖 Hastebin AI - Conversational Usage Examples\n")
    print("=" * 70)
    print("Demonstrating how AI assistants interact with Hastebin in conversations")
    print("=" * 70)
    
    # Check API connection
    ai = HastebinAI()
    
    print("\n📡 Checking API connection...")
    try:
        response = requests.get(f"{ai.base_url}/health")
        if response.status_code == 200:
            print(f"   ✅ API is running at {ai.base_url}")
        else:
            print(f"   ⚠️  API responded with status {response.status_code}")
    except Exception as e:
        print(f"   ❌ Could not connect to API: {e}")
        print(f"   Make sure the API is running: cargo run --bin hastebin")
        exit(1)
    
    print("\n" + "=" * 70)
    print("\n🎬 SCENARIO 1: User shares a paste and asks questions")
    print("-" * 70)
    scenario_user_shares_link()
    
    print("\n" + "=" * 70)
    print("\n🎬 SCENARIO 2: User asks for code improvements")
    print("-" * 70)
    scenario_improve_code()
    
    print("\n" + "=" * 70)
    print("\n🎬 SCENARIO 3: User requests code review with comments")
    print("-" * 70)
    scenario_add_review_comments()
    
    print("\n" + "=" * 70)
    print("\n🎬 SCENARIO 4: User asks about existing feedback")
    print("-" * 70)
    scenario_read_existing_comments()
    
    print("\n" + "=" * 70)
    print("\n🎬 SCENARIO 5: AI creates multi-file project")
    print("-" * 70)
    scenario_multifile_sharing()
    
    print("\n" + "=" * 70)
    print("\n✅ All scenarios complete!")
    print("\n📚 Key Takeaways for AI Assistants:")
    print("   1. Extract hex IDs from shared URLs: extract_id_from_url()")
    print("   2. Convert hex to decimal before API calls: int(hex_id, 16)")
    print("   3. Convert decimal to hex when sharing: hex(decimal_id)[2:]")
    print("   4. Always check if comments are enabled before adding them")
    print("   5. Use forking to create improved versions")
    print("\n📖 For more information:")
    print(f"   • GET {ai.base_url}/openapi.json - Full API specification")
    print(f"   • GET {ai.base_url}/ai-instructions - Detailed AI guide")
    print(f"   • GET {ai.base_url}/.well-known/ai-plugin.json - Plugin manifest")
