from langchain_openai import ChatOpenAI
from langchain_core.tools import tool
from langchain_core.messages import HumanMessage
import tool_use

@tool
def sum_as_string(a: int, b: int) -> str:
    """Returns the sum of two numbers as a string."""
    print(f"{a} + {b} = {tool_use.sum_as_string(a, b)}")
    return tool_use.sum_as_string(a, b)
@tool
def minus(a: int, b: int) -> str:
    """Returns the difference of two numbers as a string."""
    print(f"{a} - {b} = {tool_use.subtract_as_string(a, b)}")
    return tool_use.subtract_as_string(a, b)


# Initialize the OpenAI model
llm = ChatOpenAI(model="gpt-3.5-turbo")

# Define the tools
@tool
def add(a: int, b: int) -> int:
    """Add two numbers together."""
    result = a + b
    print(f"{a} + {b} = {result}")
    return result

@tool
def subtract(a: int, b: int) -> int:
    """Subtract b from a."""
    result = a - b
    print(f"{a} - {b} = {result}")
    return result

def main():
    print(tool_use.sum_as_string(1, 2))
    # Bind the tools to the model
    llm_with_tools = llm.bind_tools([sum_as_string, minus])
    
    print("Math Tools LLM")
    print("Type 'exit' to quit")
    print("Example: What is 5 plus 3?")
    
    while True:
        user_input = input("\nYou: ")
        if user_input.lower() == 'exit':
            break
            
        # Get the model's response with tool calls
        message = llm_with_tools.invoke([("user", user_input)])
        
        # Execute any tool calls
        if hasattr(message, 'tool_calls') and message.tool_calls:
            for tool_call in message.tool_calls:
                tool_name = tool_call['name']
                args = tool_call['args']
                
                if tool_name == 'sum_as_string':
                    print(sum_as_string.invoke(args))
                elif tool_name == 'minus':
                    print(minus.invoke(args))

if __name__ == "__main__":
    main()
