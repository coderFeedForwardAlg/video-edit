import json
from typing import Dict, List, Any, Optional
from langchain_ollama import ChatOllama
from langchain_core.messages import HumanMessage, AIMessage
from langchain_core.prompts import ChatPromptTemplate
from langchain_core.output_parsers import StrOutputParser

import tool_use

def sum_as_string(a: int, b: int) -> str:
    """Returns the sum of two numbers as a string."""
    return tool_use.sum_as_string(a, b)

# Initialize the LLM
llm = ChatOllama(
    model="llama3",
    temperature=0
)

def extract_numbers(text: str) -> tuple[Optional[int], Optional[int]]:
    """Extract two integers from a text string."""
    import re
    numbers = [int(num) for num in re.findall(r'\d+', text)]
    if len(numbers) >= 2:
        return numbers[0], numbers[1]
    return None, None

def run_agent(question: str) -> str:
    """Run the agent with the given question and return the final response."""
    try:
        # First, try to extract numbers from the question
        a, b = extract_numbers(question)
        
        if a is not None and b is not None:
            # If we found two numbers, use the tool directly
            result = sum_as_string(a, b)
            return f"The sum of {a} and {b} is {result}"
        
        # If we couldn't extract numbers, ask the LLM to help
        prompt = ChatPromptTemplate.from_messages([
            ("system", """You are a helpful AI assistant that helps with calculations. 
            When asked to add two numbers, extract them from the question and provide the result.
            If the question doesn't contain two numbers, ask for clarification."""),
            ("human", "{question}")
        ])
        
        chain = prompt | llm | StrOutputParser()
        response = chain.invoke({"question": question})
        
        # Check if the response contains numbers and try to calculate
        a, b = extract_numbers(response)
        if a is not None and b is not None:
            result = sum_as_string(a, b)
            return f"The sum of {a} and {b} is {result}"
            
        return response
        
    except Exception as e:
        return f"An error occurred: {str(e)}"

# Example usage
if __name__ == "__main__":
    questions = [
        "What is 5 plus 7?",
        "Add 10 and 20",
        "Calculate the sum of 15 and 25",
        "How much is 100 plus 200?",
        "I need to add 50 and 75"
    ]
    
    for question in questions:
        print(f"\nQuestion: {question}")
        response = run_agent(question)
        print(f"Response: {response}")
    
    # Test with a non-math question
    print("\nQuestion: What's the weather like today?")
    print(f"Response: {run_agent("What's the weather like today?")}")
