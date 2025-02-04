# Widgets

Implementing widgets with state is hard to learn. Being able to maintain state between frames is hard. 
This document is meant to outline the process for another widget.

1. Determine data what data is passed by reference and state
   1. State is maintained by the widget and is normally not user facing. Ex: contents of an editbox
   2. Referenced data is passed in at init time during the normal life cycle.
2. Create a struct for maintaining state
   1. This will implement functions to load, and store state on the Context object maintained by egui
3. Create a struct for maintaining widget 
   1. This will build with references to user provided data.
   2. During the lifecycle of this widget, load the state object by id
4. Continue the normal life cycle