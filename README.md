# game-of-life-in-bevy
spaggeti code but it works.

press '/' to change rulestrings(will cause a prompt to appear in the terminal), press space to pause. You can only place/delete cells when paused. 

rulestring works as follows.

"/birth/survive/corpse lifetime"

example: 3/23/0 - exactly three neighbors to be born if dead, 
must have exactly two or three neighbors to continue on to the next frame, 
and 0 decay states.

a decaying cell cannot be filled with a new live cell until it has fully decayed, leaving a dead/empty cell. 

example: 2/345/4 - exactly two neighbors to be born if dead, 
must have exactly three, four, or five neighbors to continue on to the next frame, 
and 4 decay states.

a decaying cell cannot be filled with a new live cell until it has fully decayed, leaving a dead/empty cell. 

