import requests
import re

def fetch_php_file():
    url = "https://raw.githubusercontent.com/goto-bus-stop/recanalyst/refs/heads/master/resources/lang/en/ageofempires.php"
    response = requests.get(url)
    if response.status_code == 200:
        print(f"Download successful. Downloaded {len(response.text)} bytes.")
    else:
        print(f"Download failed with status code {response.status_code}.")
    return response.text

def extract_arrays(content):
    # Find all array definitions
    arrays = {}
    current_key = None
    current_array = []
    
    for line in content.split('\n'):
        line = line.strip()
        
        # Match array key definition
        if "=>" in line and "array (" not in line:
            key_match = re.match(r"([^\s]+)\s*=>\s*'((?:[^'\\]|\\'|\\\\)*)'", line)
            if key_match and current_key:
                current_array.append((int(key_match.group(1)), key_match.group(2)))
        
        # Match start of new array
        array_start = re.match(r"'([^']+)'\s*=>\s*$", line)
        if array_start:
            if current_key and current_array:
                arrays[current_key] = current_array
            current_key = array_start.group(1)
            current_array = []
            
    if current_key and current_array:
        arrays[current_key] = current_array
        
    return arrays

def generate_rust_code(arrays):
    rust_code = """use phf::phf_map;

"""
    
    for key, values in arrays.items():
        if values:  # Only process non-empty arrays
            const_name = f"{key.upper()}_TRANS"
            rust_code += f"pub static {const_name}: phf::Map<i32, &'static str> = phf_map! {{\n"
            for num, text in values:
                rust_code += f"    {num}i32 => \"{text}\",\n"
            rust_code += "};\n\n"
    
    return rust_code

def main():
    content = fetch_php_file()
    arrays = extract_arrays(content)
    rust_code = generate_rust_code(arrays)
    
    with open("translations.rs", "w", encoding="utf-8") as f:
        f.write(rust_code)

if __name__ == "__main__":
    main()