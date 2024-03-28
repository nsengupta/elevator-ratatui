
#[derive(Debug)]
pub struct CarriageData{
    current_floor: u8,
    dest_floor: Option<u8>,
    next_dests_waiting_list: Vec<u8>,
    mx_floors: u16,
    emergency_op_requested: bool
}

impl CarriageData {
    pub fn new(
        mx_floors: u16, 
    ) -> Self {
            CarriageData {
                current_floor: 0,
                dest_floor: None,
                next_dests_waiting_list: Vec::new(),
                mx_floors: mx_floors - 1u16, // floors are zero-indexed, 0 to (mx_floors - 1),
                emergency_op_requested: false
            }
    }
    pub fn where_is(&self) -> u8 {
        self.current_floor
    }

    pub fn already_at_floor(&self, floor_index: u8) -> bool {
        self.current_floor == floor_index
    }

    pub fn set_next_destination(&mut self, dest_floor: u8) -> u8 {
        if dest_floor as u16 <= self.mx_floors {
            self.dest_floor = Some(dest_floor);
        }
        self.current_floor
    }

    pub fn enqueue_next_destination(&mut self, floor_id: u8) -> () {
        self.next_dests_waiting_list.push(floor_id);
    }

    pub fn on_arrival(&mut self) -> u8 {
        self.current_floor = self.dest_floor.take().unwrap();
        self.current_floor
    }

    pub fn any_destination_in_queue(&self) -> bool  {
        !self.next_dests_waiting_list.is_empty()
    }

    pub fn dequeue_next_destination(&mut self) -> Option<u8> {
        let head = 
            if !self.next_dests_waiting_list.is_empty() {
                    Some(self.next_dests_waiting_list.remove(0))
            }
            else { None };
        head    
    }

    pub fn prepare_for_emergency(&mut self) -> () {
        self.emergency_op_requested = true;
    }

    pub fn is_emergency_op_requested(&self) -> bool  {
        self.emergency_op_requested
    }


}

mod tests {
    use super::*;

    #[test]
    fn when_initialized_then_no_dest () -> () {

        let carriage = CarriageData::new(8);

        assert_eq!(carriage.current_floor, 0);
        assert_eq!(carriage.dest_floor, None);
    }

    #[test]
    fn when_multiple_dest_specified_then_next_dest_should_be_in_order_of_specification() {

        use rand::seq::SliceRandom;

        // Create a vector of u8
        let mut data: Vec<u8> = (0..8).collect();

        // Shuffle the contents randomly
        let mut rng = rand::thread_rng();
        data.shuffle(&mut rng);

        let carriage = &mut CarriageData::new(8);

        for i in data.iter() {
            carriage.enqueue_next_destination(*i);
        }

        for i in (0..8).into_iter() {
            assert_eq!(data[i as usize], carriage.dequeue_next_destination().unwrap_or(8));
        }

    }

    #[test]
    fn when_dest_is_beyond_either_ends_then_dest_floor_should_remain_the_same() {

        let carriage = &mut CarriageData::new(8);

        carriage.set_next_destination(5);

        assert_eq!(carriage.current_floor,0);
        assert_eq!(carriage.dest_floor.unwrap(),   5);

        carriage.set_next_destination(8);

        assert_eq!(carriage.current_floor,0);
        assert_eq!(carriage.dest_floor.unwrap(),   5);
    }

}