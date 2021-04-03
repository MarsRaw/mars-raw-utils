/*
    Attempt at hot pixel detection and removal. 
    Initial thought would be:
      - Create black imagebuffer
      - Detect pixel, mark it as white
      - Use the imagebuffer as an inpaint mask
      - ???
      - Profit!
*/