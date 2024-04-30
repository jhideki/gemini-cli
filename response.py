def printSubArrays(arr, n):
    # Create an empty hashmap
    h = {}
    
    # Insert the first element with sum 0
    h[0] = -1
    
    # Iterate through the array
    sum = 0
    for i in range(n):
        # Add the current element to the sum
        sum += arr[i]
        
        # Check if the sum is already present in the hashmap
        if sum in h:
            # Print the subarray from the index stored in the hashmap to the current index
            print("Subarray with sum 0 from index %d to %d" % (h[sum] + 1, i))
        
        # Insert the sum into the hashmap
        h[sum] = i
    
    # Driver code
if __name__=='__main__':
    arr = [1, 4, 20, 3, 10, 5]
    n = len(